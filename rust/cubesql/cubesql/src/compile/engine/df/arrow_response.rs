//! Reads a Cube response that arrives as Arrow (a query served by Cube Store)
//! without converting it value by value first.
//!
//! Without this, every value of such a response is turned into a generic value
//! by the orchestrator, then into a [`FieldValue`], and only then into the
//! Arrow builder of its schema field. That chain decides what each value
//! becomes: numbers pass through `f64`, decimals and timestamps through
//! strings. The functions here produce exactly the same arrays, either by
//! feeding the same builders [`FieldValue`]s made straight from the Arrow
//! values ([`arrow_field_values`]), or by computing the result directly for
//! the common cases ([`convert_arrow_column`]).
//!
//! A streamed response is read differently: there every value reaches the
//! builders as the text the orchestrator writes for it, numbers included.
//! [`arrow_text_values`] and [`convert_arrow_column_as_text`] are the same pair
//! for that path.

use crate::{compile::engine::df::scan::FieldValue, CubeError};
use chrono::{DateTime, NaiveDateTime};
use datafusion::arrow::{
    array::{
        Array, ArrayRef, BinaryArray, BooleanArray, Date32Array, Date64Array, DecimalArray,
        DecimalBuilder, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
        LargeStringArray, StringArray, TimestampMicrosecondArray, TimestampMillisecondArray,
        TimestampNanosecondArray, TimestampSecondArray, UInt16Array, UInt32Array, UInt64Array,
        UInt8Array,
    },
    compute::concat,
    datatypes::{DataType, SchemaRef, TimeUnit},
    ipc::{self, reader::StreamReader},
};
use std::{borrow::Cow, io::Cursor, sync::Arc, sync::OnceLock};

/// For readers of responses outside this crate, which has the Arrow they need.
pub use datafusion::arrow::{
    array::ArrayRef as ResponseArray, datatypes::DataType as ResponseType,
};

/// Whether Cube Store responses are read as Arrow (`CUBESQL_ARROW_PASSTHROUGH`).
pub fn arrow_passthrough_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();

    *ENABLED.get_or_init(|| {
        std::env::var("CUBESQL_ARROW_PASSTHROUGH")
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "true" | "1"))
            .unwrap_or(false)
    })
}

/// Reads an Arrow IPC stream into one array per column. `None` for a stream
/// with a column this version of Arrow can't read correctly ([`readable_schema`]).
pub fn read_arrow_ipc(bytes: &[u8]) -> Result<Option<(SchemaRef, Vec<ArrayRef>)>, CubeError> {
    if !readable_schema(bytes) {
        return Ok(None);
    }

    let reader = StreamReader::try_new(Cursor::new(bytes), None)?;
    let schema = reader.schema();
    let batches = reader.collect::<Result<Vec<_>, _>>()?;

    let columns = (0..schema.fields().len())
        .map(|idx| match batches.as_slice() {
            [batch] => Ok(batch.column(idx).clone()),
            _ => {
                let parts: Vec<&dyn Array> =
                    batches.iter().map(|b| b.column(idx).as_ref()).collect();
                Ok(concat(&parts)?)
            }
        })
        .collect::<Result<Vec<_>, CubeError>>()?;

    Ok(Some((schema, columns)))
}

/// Whether every column of the stream has a type this version of Arrow reads
/// as the writer meant it. Cube Store writes with a newer Arrow, and the
/// reader here reads a 256-bit decimal as a 128-bit one, and panics on types
/// added since. Reads the schema message, the stream's first, as written.
fn readable_schema(bytes: &[u8]) -> bool {
    let meta = match bytes {
        // A continuation marker, then the metadata length
        [0xff, 0xff, 0xff, 0xff, a, b, c, d, rest @ ..] => {
            let len = i32::from_le_bytes([*a, *b, *c, *d]);
            (len >= 0).then(|| rest.get(..len as usize)).flatten()
        }
        _ => None,
    };
    let Some(schema) = meta
        .and_then(|meta| ipc::root_as_message(meta).ok())
        .and_then(|message| message.header_as_schema())
    else {
        return false;
    };

    schema.fields().map_or(true, |fields| {
        fields.iter().all(|field| {
            let plain = field.dictionary().is_none()
                && field
                    .children()
                    .map_or(true, |children| children.is_empty());
            plain
                && match field.type_type() {
                    ipc::Type::Null
                    | ipc::Type::Bool
                    | ipc::Type::Int
                    | ipc::Type::Utf8
                    | ipc::Type::LargeUtf8
                    | ipc::Type::Binary
                    | ipc::Type::Date => true,
                    ipc::Type::FloatingPoint => field
                        .type_as_floating_point()
                        .is_some_and(|f| f.precision() != ipc::Precision::HALF),
                    ipc::Type::Decimal => {
                        field.type_as_decimal().is_some_and(|d| d.bitWidth() == 128)
                    }
                    ipc::Type::Timestamp => field
                        .type_as_timestamp()
                        .is_some_and(|t| t.timezone().is_none()),
                    _ => false,
                }
        })
    })
}

/// `array` converted to `data_type`, for the cases computed directly. `None`
/// for any other case, which [`arrow_field_values`] covers.
pub fn convert_arrow_column(
    array: &ArrayRef,
    data_type: &DataType,
) -> Result<Option<ArrayRef>, CubeError> {
    let converted: Option<ArrayRef> = match (array.data_type(), data_type) {
        (DataType::Utf8, DataType::Utf8) | (DataType::Boolean, DataType::Boolean) => {
            Some(array.clone())
        }
        (source, DataType::Float64) if is_number(source) => {
            let values = numbers(array.as_ref()).expect("a number type");
            Some(Arc::new(values.collect::<Float64Array>()))
        }
        // As `append_number` rounds a number to an integer
        (source, DataType::Int64) if is_number(source) => {
            let values = numbers(array.as_ref()).expect("a number type");
            Some(Arc::new(
                values
                    .map(|v| v.map(|n| n.round() as i64))
                    .collect::<Int64Array>(),
            ))
        }
        (DataType::Decimal(_, scale), DataType::Float64) => {
            decimal_to_f64(array, *scale).map(|a| Arc::new(a) as ArrayRef)
        }
        (DataType::Decimal(_, scale), DataType::Decimal(precision, target_scale)) => {
            rescale_decimal(array, *scale, *precision, *target_scale)?
                .map(|a| Arc::new(a) as ArrayRef)
        }
        (DataType::Timestamp(unit, None), DataType::Timestamp(TimeUnit::Nanosecond, None)) => {
            timestamps_millis(array, unit).map(|millis| {
                Arc::new(
                    millis
                        .into_iter()
                        .map(|v| v.and_then(|ms| ms.checked_mul(1_000_000)))
                        .collect::<TimestampNanosecondArray>(),
                ) as ArrayRef
            })
        }
        (DataType::Timestamp(unit, None), DataType::Timestamp(TimeUnit::Millisecond, None)) => {
            timestamps_millis(array, unit).map(|millis| {
                Arc::new(millis.into_iter().collect::<TimestampMillisecondArray>()) as ArrayRef
            })
        }
        _ => None,
    };

    Ok(converted)
}

/// `array` converted to `data_type` as its text would be, for the cases
/// computed directly. Text holds an integer's every digit, so integers go into
/// `Int64` exactly, not through `f64`. Text that isn't a whole number reads as
/// NULL into an integer, where a number would be rounded, so floats and
/// unsigned integers that may not fit are left to [`arrow_text_values`]. The
/// other cases of [`convert_arrow_column`] read a value's text as it reads the
/// value.
pub fn convert_arrow_column_as_text(
    array: &ArrayRef,
    data_type: &DataType,
) -> Result<Option<ArrayRef>, CubeError> {
    macro_rules! to_int64 {
        ($ty:ty) => {{
            let a = array.as_any().downcast_ref::<$ty>().expect("its own type");
            Ok(Some(Arc::new(
                a.iter().map(|v| v.map(i64::from)).collect::<Int64Array>(),
            )))
        }};
    }

    match (array.data_type(), data_type) {
        (DataType::Int64, DataType::Int64) => Ok(Some(array.clone())),
        (DataType::Int8, DataType::Int64) => to_int64!(Int8Array),
        (DataType::Int16, DataType::Int64) => to_int64!(Int16Array),
        (DataType::Int32, DataType::Int64) => to_int64!(Int32Array),
        (DataType::UInt8, DataType::Int64) => to_int64!(UInt8Array),
        (DataType::UInt16, DataType::Int64) => to_int64!(UInt16Array),
        (DataType::UInt32, DataType::Int64) => to_int64!(UInt32Array),
        (_, DataType::Int64) => Ok(None),
        _ => convert_arrow_column(array, data_type),
    }
}

/// The values of `array` as the text the orchestrator writes for them in a
/// streamed response, or `None` for an array type it has no conversion for
/// here. Text is what that response holds for every value but a null.
pub fn arrow_text_values<'a>(
    array: &'a ArrayRef,
) -> Option<Box<dyn Iterator<Item = Result<FieldValue<'a>, CubeError>> + 'a>> {
    let len = array.len();

    macro_rules! texts {
        ($ty:ty, |$v:ident| $text:expr) => {{
            let a = array.as_any().downcast_ref::<$ty>()?;
            Box::new((0..len).map(move |i| {
                Ok(if a.is_null(i) {
                    FieldValue::Null
                } else {
                    let $v = a.value(i);
                    FieldValue::String(Cow::Owned($text))
                })
            }))
        }};
    }

    Some(match array.data_type() {
        DataType::Int8 => texts!(Int8Array, |v| v.to_string()),
        DataType::Int16 => texts!(Int16Array, |v| v.to_string()),
        DataType::Int32 => texts!(Int32Array, |v| v.to_string()),
        DataType::Int64 => texts!(Int64Array, |v| v.to_string()),
        DataType::UInt8 => texts!(UInt8Array, |v| v.to_string()),
        DataType::UInt16 => texts!(UInt16Array, |v| v.to_string()),
        DataType::UInt32 => texts!(UInt32Array, |v| v.to_string()),
        DataType::UInt64 => texts!(UInt64Array, |v| v.to_string()),
        DataType::Float32 => texts!(Float32Array, |v| (v as f64).to_string()),
        DataType::Float64 => texts!(Float64Array, |v| v.to_string()),
        DataType::Boolean => texts!(BooleanArray, |v| v.to_string()),
        // The rest are text in both responses
        _ => arrow_field_values(array)?,
    })
}

/// The values of `array` as the [`FieldValue`]s the orchestrator's generic
/// values become, or `None` for an array type it has no conversion for here.
pub fn arrow_field_values<'a>(
    array: &'a ArrayRef,
) -> Option<Box<dyn Iterator<Item = Result<FieldValue<'a>, CubeError>> + 'a>> {
    let array = array.as_ref();
    let len = array.len();

    macro_rules! values {
        ($ty:ty, |$a:ident, $i:ident| $value:expr) => {{
            let $a = array.as_any().downcast_ref::<$ty>()?;
            Box::new((0..len).map(move |$i| {
                Ok(if $a.is_null($i) {
                    FieldValue::Null
                } else {
                    $value
                })
            }))
        }};
    }

    if is_number(array.data_type()) {
        let values = numbers(array)?;
        return Some(Box::new(
            values.map(|v| Ok(v.map(FieldValue::Number).unwrap_or(FieldValue::Null))),
        ));
    }

    Some(match array.data_type() {
        DataType::Null => Box::new((0..len).map(|_| Ok(FieldValue::Null))),
        DataType::Boolean => values!(BooleanArray, |a, i| FieldValue::Bool(a.value(i))),
        DataType::Utf8 => values!(StringArray, |a, i| FieldValue::String(Cow::Borrowed(
            a.value(i)
        ))),
        DataType::LargeUtf8 => values!(LargeStringArray, |a, i| FieldValue::String(Cow::Borrowed(
            a.value(i)
        ))),
        DataType::Binary => values!(BinaryArray, |a, i| FieldValue::String(Cow::Owned(
            hex_upper(a.value(i))
        ))),
        DataType::Decimal(_, scale) => {
            let scale = *scale;
            values!(DecimalArray, |a, i| FieldValue::String(Cow::Owned(
                decimal_to_string(a.value(i), scale)
            )))
        }
        DataType::Timestamp(_, None) | DataType::Date32 | DataType::Date64 => {
            Box::new((0..len).map(move |i| {
                Ok(match datetime(array, i) {
                    Some(dt) => FieldValue::String(Cow::Owned(
                        dt.format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
                    )),
                    None => FieldValue::Null,
                })
            }))
        }
        _ => return None,
    })
}

/// Bytes as the orchestrator writes them: `0x` and upper case hex.
fn hex_upper(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 + bytes.len() * 2);
    s.push_str("0x");
    for b in bytes {
        s.push_str(&format!("{:02X}", b));
    }
    s
}

/// Number types, read as `f64` as the orchestrator's numbers are.
fn is_number(data_type: &DataType) -> bool {
    matches!(
        data_type,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float32
            | DataType::Float64
    )
}

fn numbers<'a>(array: &'a dyn Array) -> Option<Box<dyn Iterator<Item = Option<f64>> + 'a>> {
    macro_rules! numbers {
        ($ty:ty) => {{
            let a = array.as_any().downcast_ref::<$ty>()?;
            Box::new(a.iter().map(|v| v.map(|n| n as f64)))
        }};
    }

    Some(match array.data_type() {
        DataType::Int8 => numbers!(Int8Array),
        DataType::Int16 => numbers!(Int16Array),
        DataType::Int32 => numbers!(Int32Array),
        DataType::Int64 => numbers!(Int64Array),
        DataType::UInt8 => numbers!(UInt8Array),
        DataType::UInt16 => numbers!(UInt16Array),
        DataType::UInt32 => numbers!(UInt32Array),
        DataType::UInt64 => numbers!(UInt64Array),
        DataType::Float32 => numbers!(Float32Array),
        DataType::Float64 => numbers!(Float64Array),
        _ => return None,
    })
}

/// The value at `i` as the orchestrator reads a date or timestamp: `None` for
/// a null, and for a value chrono can't represent.
fn datetime(array: &dyn Array, i: usize) -> Option<NaiveDateTime> {
    if array.is_null(i) {
        return None;
    }

    let any = array.as_any();
    let (secs, nanos) = match array.data_type() {
        DataType::Timestamp(TimeUnit::Second, _) => {
            (any.downcast_ref::<TimestampSecondArray>()?.value(i), 0)
        }
        DataType::Timestamp(TimeUnit::Millisecond, _) => split(
            any.downcast_ref::<TimestampMillisecondArray>()?.value(i),
            1_000,
        ),
        DataType::Timestamp(TimeUnit::Microsecond, _) => split(
            any.downcast_ref::<TimestampMicrosecondArray>()?.value(i),
            1_000_000,
        ),
        DataType::Timestamp(TimeUnit::Nanosecond, _) => split(
            any.downcast_ref::<TimestampNanosecondArray>()?.value(i),
            1_000_000_000,
        ),
        DataType::Date32 => (
            (any.downcast_ref::<Date32Array>()?.value(i) as i64).checked_mul(86_400)?,
            0,
        ),
        DataType::Date64 => split(any.downcast_ref::<Date64Array>()?.value(i), 1_000),
        _ => return None,
    };

    DateTime::from_timestamp(secs, nanos).map(|dt| dt.naive_utc())
}

/// Splits a count of `per_second` units into whole seconds and nanoseconds.
fn split(value: i64, per_second: i64) -> (i64, u32) {
    let nanos = value.rem_euclid(per_second) * (1_000_000_000 / per_second);
    (value.div_euclid(per_second), nanos as u32)
}

/// Milliseconds of each timestamp, as reading it back from the orchestrator's
/// `%.3f` string gives: floored to the millisecond. `None` when a value falls
/// outside the years that string round-trips, years 1 to 9999.
fn timestamps_millis(array: &ArrayRef, unit: &TimeUnit) -> Option<Vec<Option<i64>>> {
    // Seconds from 0001-01-01 and to 9999-12-31T23:59:59
    const MIN_SECS: i64 = -62_135_596_800;
    const MAX_SECS: i64 = 253_402_300_799;

    let per_second = match unit {
        TimeUnit::Second => 1,
        TimeUnit::Millisecond => 1_000,
        TimeUnit::Microsecond => 1_000_000,
        TimeUnit::Nanosecond => 1_000_000_000,
    };

    (0..array.len())
        .map(|i| {
            if array.is_null(i) {
                return Some(None);
            }

            let any = array.as_any();
            let value = match unit {
                TimeUnit::Second => any.downcast_ref::<TimestampSecondArray>()?.value(i),
                TimeUnit::Millisecond => any.downcast_ref::<TimestampMillisecondArray>()?.value(i),
                TimeUnit::Microsecond => any.downcast_ref::<TimestampMicrosecondArray>()?.value(i),
                TimeUnit::Nanosecond => any.downcast_ref::<TimestampNanosecondArray>()?.value(i),
            };
            let secs = value.div_euclid(per_second);
            if !(MIN_SECS..=MAX_SECS).contains(&secs) {
                return None;
            }

            let millis_in_second = value.rem_euclid(per_second) * 1_000 / per_second;
            Some(Some(secs * 1_000 + millis_in_second))
        })
        .collect()
}

/// Decimals as `f64`, as parsing their strings gives. Dividing the mantissa by
/// a power of ten rounds once, and so gives the nearest `f64` to the decimal,
/// as parsing does, while both are exact in `f64`: a mantissa below 2^53 and a
/// power up to 10^22. `None` when a value is outside that.
fn decimal_to_f64(array: &ArrayRef, scale: usize) -> Option<Float64Array> {
    const MAX_EXACT: i128 = 1 << 53;

    let a = array.as_any().downcast_ref::<DecimalArray>()?;
    if scale > 22 {
        return None;
    }
    let divisor = 10f64.powi(scale as i32);

    (0..a.len())
        .map(|i| {
            if a.is_null(i) {
                return Some(None);
            }
            let m = a.value(i);
            (-MAX_EXACT < m && m < MAX_EXACT).then(|| Some(m as f64 / divisor))
        })
        .collect::<Option<Vec<_>>>()
        .map(Float64Array::from)
}

/// Decimals at another precision and scale, as `append_decimal` reads their
/// strings. `None` when a value would hit one of its errors (more digits than
/// the precision or scale allow), which the string path then reports.
fn rescale_decimal(
    array: &ArrayRef,
    scale: usize,
    precision: usize,
    target_scale: usize,
) -> Result<Option<DecimalArray>, CubeError> {
    let Some(a) = array.as_any().downcast_ref::<DecimalArray>() else {
        return Ok(None);
    };
    let (Some(unit), Some(target_unit)) = (
        10i128.checked_pow(scale as u32),
        10i128.checked_pow(target_scale as u32),
    ) else {
        return Ok(None);
    };

    let mut builder = DecimalBuilder::new(a.len(), precision, target_scale);
    for i in 0..a.len() {
        if a.is_null(i) {
            builder.append_null()?;
            continue;
        }

        let m = a.value(i);
        let Some(abs) = m.checked_abs() else {
            return Ok(None);
        };
        let (int_part, frac) = (abs / unit, abs % unit);

        // The string has no trailing zeros in its fraction
        let frac_digits = if frac == 0 {
            0
        } else {
            let mut f = frac;
            let mut zeros = 0;
            while f % 10 == 0 {
                f /= 10;
                zeros += 1;
            }
            scale - zeros
        };
        if frac_digits > target_scale {
            return Ok(None);
        }

        // Sign, integer digits ("0" below one) and the padded fraction
        let int_digits = int_part.checked_ilog10().map_or(1, |d| d as usize + 1);
        if (m < 0) as usize + int_digits + target_scale > precision {
            return Ok(None);
        }

        let frac_scaled = if target_scale >= scale {
            frac * 10i128.pow((target_scale - scale) as u32)
        } else {
            frac / 10i128.pow((scale - target_scale) as u32)
        };
        let Some(value) = int_part
            .checked_mul(target_unit)
            .and_then(|v| v.checked_add(frac_scaled))
        else {
            return Ok(None);
        };

        builder.append_value(if m < 0 { -value } else { value })?;
    }

    Ok(Some(builder.finish()))
}

/// A decimal as the orchestrator formats it: `scale` fractional digits with
/// trailing zeros removed. Mirrors `decimal_to_string` in `cubeorchestrator`.
fn decimal_to_string(mantissa: i128, scale: usize) -> String {
    let raw = mantissa.to_string();
    if scale == 0 {
        return raw;
    }

    let (sign, digits) = match raw.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", raw.as_str()),
    };

    let (int_part, frac) = if digits.len() > scale {
        let (int_part, frac) = digits.split_at(digits.len() - scale);
        (int_part, frac.to_string())
    } else {
        (
            "0",
            format!("{}{}", "0".repeat(scale - digits.len()), digits),
        )
    };

    let frac = frac.trim_end_matches('0');
    if frac.is_empty() {
        format!("{sign}{int_part}")
    } else {
        format!("{sign}{int_part}.{frac}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::engine::df::scan::{transform_response, ColumnarValueObject, MemberField};
    use datafusion::arrow::{
        array::{DecimalBuilder, TimestampMicrosecondArray},
        datatypes::{Field, Schema},
        record_batch::RecordBatch,
    };

    /// A response holding one Arrow column, `a`, read either as Arrow or as
    /// the values the orchestrator would have produced.
    struct Response {
        array: ArrayRef,
        as_arrow: bool,
    }

    impl ColumnarValueObject for Response {
        fn len(&mut self) -> Result<usize, CubeError> {
            Ok(self.array.len())
        }

        fn arrow_column(&mut self, _field_name: &str) -> Result<Option<ArrayRef>, CubeError> {
            Ok(self.as_arrow.then(|| self.array.clone()))
        }

        fn column<'a>(
            &'a mut self,
            _field_name: &str,
        ) -> Result<Box<dyn Iterator<Item = Result<FieldValue<'a>, CubeError>> + 'a>, CubeError>
        {
            Ok(arrow_field_values(&self.array).expect("a supported type"))
        }
    }

    fn read(array: &ArrayRef, data_type: &DataType, as_arrow: bool) -> Result<RecordBatch, String> {
        let schema = Arc::new(Schema::new(vec![Field::new("a", data_type.clone(), true)]));
        let mut response = Response {
            array: array.clone(),
            as_arrow,
        };
        transform_response(
            &mut response,
            schema,
            &[MemberField::regular("a".to_string())],
        )
        .map_err(|e| e.to_string())
    }

    /// Reading the column as Arrow gives what reading its values gives, error included.
    fn assert_same(array: ArrayRef, data_types: &[DataType]) {
        for data_type in data_types {
            let expected = read(&array, data_type, false);
            let actual = read(&array, data_type, true);
            // Compared as data, not Debug, which panics on years past 9999
            let same = match (&actual, &expected) {
                (Ok(a), Ok(e)) => a.column(0).data() == e.column(0).data(),
                (Err(a), Err(e)) => a == e,
                _ => false,
            };
            assert!(
                same,
                "{:?} read as {:?}: {}",
                array.data_type(),
                data_type,
                match (&actual, &expected) {
                    (Err(a), Err(e)) => format!("errors {:?} vs {:?}", a, e),
                    (Err(a), _) => format!("only Arrow failed: {}", a),
                    (_, Err(e)) => format!("only values failed: {}", e),
                    _ => "different data".to_string(),
                }
            );
        }
    }

    fn decimals(values: &[Option<i128>], precision: usize, scale: usize) -> ArrayRef {
        let mut builder = DecimalBuilder::new(values.len(), precision, scale);
        for v in values {
            match v {
                Some(v) => builder.append_value(*v).unwrap(),
                None => builder.append_null().unwrap(),
            }
        }
        Arc::new(builder.finish())
    }

    const NUMBER_TARGETS: &[DataType] = &[
        DataType::Float64,
        DataType::Float32,
        DataType::Int64,
        DataType::Int32,
        DataType::Int16,
        DataType::UInt64,
        DataType::Utf8,
        DataType::Boolean,
    ];

    #[test]
    fn integers() {
        let values = vec![
            Some(0),
            Some(-1),
            Some(42),
            Some((1 << 53) + 1),
            Some(i64::MAX),
            Some(i64::MIN),
            None,
        ];
        assert_same(Arc::new(Int64Array::from(values)), NUMBER_TARGETS);
        assert_same(
            Arc::new(Int32Array::from(vec![Some(7), None, Some(-7)])),
            NUMBER_TARGETS,
        );
        assert_same(
            Arc::new(UInt64Array::from(vec![Some(u64::MAX), None, Some(3)])),
            NUMBER_TARGETS,
        );
    }

    #[test]
    fn floats() {
        let values = vec![
            Some(1.5),
            Some(-2.5),
            Some(0.1),
            Some(0.30000000000000004),
            Some(1e300),
            Some(-0.0),
            Some(f64::NAN),
            Some(f64::INFINITY),
            None,
        ];
        assert_same(Arc::new(Float64Array::from(values)), NUMBER_TARGETS);
        assert_same(
            Arc::new(Float32Array::from(vec![Some(0.1f32), None, Some(2.5)])),
            NUMBER_TARGETS,
        );
    }

    #[test]
    fn decimals_to_numbers() {
        let values = [
            Some(25987600),
            Some(6199200000),
            Some(-250),
            Some(25),
            Some(0),
            Some((1 << 53) - 1),
            Some(1 << 53),
            Some(i128::MAX / 7),
            None,
        ];
        assert_same(decimals(&values, 38, 5), NUMBER_TARGETS);
        assert_same(decimals(&values, 38, 0), NUMBER_TARGETS);
        assert_same(decimals(&values, 38, 30), NUMBER_TARGETS);
    }

    #[test]
    fn decimals_to_decimals() {
        let values = [
            Some(25987600),
            Some(6199200000),
            Some(-250),
            Some(-25),
            Some(25),
            Some(0),
            Some(100),
            None,
        ];
        let targets = [
            DataType::Decimal(38, 10),
            DataType::Decimal(38, 5),
            DataType::Decimal(38, 3),
            DataType::Decimal(38, 0),
            DataType::Decimal(12, 5),
            DataType::Decimal(8, 5),
            DataType::Decimal(6, 2),
            DataType::Decimal(5, 2),
        ];
        assert_same(decimals(&values, 38, 5), &targets);
        assert_same(decimals(&values, 38, 2), &targets);
        assert_same(decimals(&[Some(i128::MIN + 1), None], 38, 0), &targets);
    }

    #[test]
    fn timestamps() {
        let targets = [
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            DataType::Timestamp(TimeUnit::Millisecond, None),
            DataType::Date32,
            DataType::Utf8,
        ];
        let micros = vec![
            Some(1_700_000_000_123_456),
            Some(0),
            Some(-1),
            Some(-1_500_001),
            Some(-62_135_596_800_000_000),
            None,
        ];
        assert_same(Arc::new(TimestampMicrosecondArray::from(micros)), &targets);
        assert_same(
            Arc::new(TimestampNanosecondArray::from(vec![
                Some(1_700_000_000_123_456_789),
                Some(-999_999),
                None,
            ])),
            &targets,
        );
        assert_same(
            Arc::new(TimestampMillisecondArray::from(vec![
                Some(-1),
                Some(1_234),
                None,
            ])),
            &targets,
        );
        // Past year 9999: the string path turns these into errors or nulls
        assert_same(
            Arc::new(TimestampSecondArray::from(vec![
                Some(253_402_300_800),
                Some(1),
            ])),
            &targets,
        );
        assert_same(
            Arc::new(Date32Array::from(vec![Some(19_000), Some(-1), None])),
            &targets,
        );
    }

    #[test]
    fn strings_and_booleans() {
        assert_same(
            Arc::new(StringArray::from(vec![
                Some("a"),
                None,
                Some(""),
                Some("1.5"),
            ])),
            &[DataType::Utf8, DataType::Float64, DataType::Int64],
        );
        assert_same(
            Arc::new(StringArray::from(vec![Some("true"), Some("0"), None])),
            &[DataType::Boolean],
        );
        assert_same(
            Arc::new(BooleanArray::from(vec![Some(true), None, Some(false)])),
            &[DataType::Boolean, DataType::Utf8, DataType::Int64],
        );
    }

    /// Values read as the orchestrator formats them.
    #[test]
    fn field_values_match_the_orchestrator() {
        let strings = |array: ArrayRef| -> Vec<String> {
            arrow_field_values(&array)
                .unwrap()
                .map(|v| format!("{:?}", v.unwrap()))
                .collect()
        };

        assert_eq!(
            strings(decimals(
                &[Some(25987600), Some(6199200000), Some(-250), Some(25)],
                38,
                5
            )),
            [
                r#"String("259.876")"#,
                r#"String("61992")"#,
                r#"String("-0.0025")"#,
                r#"String("0.00025")"#
            ]
        );
        assert_eq!(
            strings(Arc::new(TimestampMicrosecondArray::from(vec![
                Some(1_700_000_000_123_456),
                Some(-1),
                None
            ]))),
            [
                r#"String("2023-11-14T22:13:20.123")"#,
                r#"String("1969-12-31T23:59:59.999")"#,
                "Null"
            ]
        );
        assert_eq!(
            strings(Arc::new(Date32Array::from(vec![Some(1)]))),
            [r#"String("1970-01-02T00:00:00.000")"#]
        );
        assert_eq!(
            strings(Arc::new(BinaryArray::from(vec![&[0xab_u8, 0x01][..]]))),
            [r#"String("0xAB01")"#]
        );
        assert_eq!(
            strings(Arc::new(Int64Array::from(vec![Some(3), None]))),
            ["Number(3.0)", "Null"]
        );
    }

    /// A stream's schema message with one decimal column of `bit_width` bits.
    fn decimal_schema_message(bit_width: i32) -> Vec<u8> {
        let mut fbb = flatbuffers::FlatBufferBuilder::new();
        let decimal = {
            let mut b = ipc::DecimalBuilder::new(&mut fbb);
            b.add_precision(38);
            b.add_scale(2);
            b.add_bitWidth(bit_width);
            b.finish()
        };
        let name = fbb.create_string("d");
        let field = {
            let mut b = ipc::FieldBuilder::new(&mut fbb);
            b.add_name(name);
            b.add_nullable(true);
            b.add_type_type(ipc::Type::Decimal);
            b.add_type_(decimal.as_union_value());
            b.finish()
        };
        let fields = fbb.create_vector(&[field]);
        let schema = {
            let mut b = ipc::SchemaBuilder::new(&mut fbb);
            b.add_fields(fields);
            b.finish()
        };
        let message = {
            let mut b = ipc::MessageBuilder::new(&mut fbb);
            b.add_version(ipc::MetadataVersion::V5);
            b.add_header_type(ipc::MessageHeader::Schema);
            b.add_header(schema.as_union_value());
            b.finish()
        };
        fbb.finish(message, None);

        let meta = fbb.finished_data();
        let mut bytes = vec![0xff; 4];
        bytes.extend((meta.len() as i32).to_le_bytes());
        bytes.extend(meta);
        bytes
    }

    #[test]
    fn reads_only_schemas_it_reads_correctly() {
        assert!(readable_schema(&decimal_schema_message(128)));
        assert!(!readable_schema(&decimal_schema_message(256)));
        assert_eq!(read_arrow_ipc(&decimal_schema_message(256)).unwrap(), None);

        let stream = |field: Field| {
            let schema = Arc::new(Schema::new(vec![field]));
            let mut bytes = vec![];
            datafusion::arrow::ipc::writer::StreamWriter::try_new(&mut bytes, &schema)
                .unwrap()
                .finish()
                .unwrap();
            bytes
        };
        let ts = |tz: Option<String>| DataType::Timestamp(TimeUnit::Microsecond, tz);
        assert!(readable_schema(&stream(Field::new("a", ts(None), true))));
        assert!(!readable_schema(&stream(Field::new(
            "a",
            ts(Some("UTC".to_string())),
            true
        ))));
        assert!(!readable_schema(&stream(Field::new(
            "a",
            DataType::List(Box::new(Field::new("item", DataType::Int64, true))),
            true
        ))));
        assert!(!readable_schema(b"not arrow"));
        assert!(!readable_schema(&[]));
    }

    #[test]
    fn reads_ipc_streams_of_several_batches() {
        use datafusion::arrow::ipc::writer::StreamWriter;

        let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int64, true)]));
        let mut bytes = vec![];
        {
            let mut writer = StreamWriter::try_new(&mut bytes, &schema).unwrap();
            for part in [vec![Some(1), None], vec![Some(3)]] {
                let batch =
                    RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(part))])
                        .unwrap();
                writer.write(&batch).unwrap();
            }
            writer.finish().unwrap();
        }

        let (read_schema, columns) = read_arrow_ipc(&bytes).unwrap().unwrap();
        assert_eq!(read_schema, schema);
        assert_eq!(
            columns[0].as_any().downcast_ref::<Int64Array>().unwrap(),
            &Int64Array::from(vec![Some(1), None, Some(3)])
        );
    }

    mod stream {
        use super::super::*;
        use crate::compile::engine::df::scan::{
            deserialize_arrow_chunk, deserialize_columnar_chunk, deserialize_text_chunk,
            stream_column_names, MemberField,
        };
        use datafusion::{
            arrow::{
                array::{DecimalBuilder, TimestampMicrosecondArray},
                datatypes::{Field, Schema},
                record_batch::RecordBatch,
            },
            scalar::ScalarValue,
        };
        use std::collections::HashMap;

        fn schema(fields: &[(&str, DataType)]) -> SchemaRef {
            Arc::new(Schema::new(
                fields
                    .iter()
                    .map(|(name, data_type)| Field::new(name, data_type.clone(), true))
                    .collect(),
            ))
        }

        /// What the rows of a chunk become today: JS writes each value's text
        /// into a JSON chunk, which cubesql reads.
        fn from_json(
            names: &[&str],
            texts: &[Vec<Option<&str>>],
            schema: SchemaRef,
            member_fields: &[MemberField],
        ) -> Result<RecordBatch, String> {
            let chunk = serde_json::json!({ "members": names, "columns": texts });
            deserialize_columnar_chunk(chunk.to_string().as_bytes(), schema, member_fields, 0)
                .map_err(|e| e.to_string())
        }

        fn same(expected: &Result<RecordBatch, String>, actual: &Result<RecordBatch, String>) {
            match (expected, actual) {
                (Ok(expected), Ok(actual)) => {
                    assert_eq!(expected.schema(), actual.schema());
                    for i in 0..expected.num_columns() {
                        assert_eq!(
                            expected.column(i).data(),
                            actual.column(i).data(),
                            "column {} differs: {:?} vs {:?}",
                            i,
                            expected.column(i),
                            actual.column(i)
                        );
                    }
                }
                (Err(expected), Err(actual)) => assert_eq!(expected, actual),
                _ => panic!("expected {:?}, got {:?}", expected, actual),
            }
        }

        /// The Arrow column `array`, whose values the orchestrator writes as
        /// `texts`, reads into each of `data_types` as those texts do.
        fn assert_same(array: ArrayRef, texts: &[Option<&str>], data_types: &[DataType]) {
            let read_texts: Vec<Option<String>> = arrow_text_values(&array)
                .expect("a supported type")
                .map(|v| match v.unwrap() {
                    FieldValue::String(s) => Some(s.into_owned()),
                    FieldValue::Null => None,
                    other => panic!("expected text, got {:?}", other),
                })
                .collect();
            assert_eq!(
                read_texts,
                texts
                    .iter()
                    .map(|t| t.map(String::from))
                    .collect::<Vec<_>>()
            );

            for data_type in data_types {
                let schema = schema(&[("m", data_type.clone())]);
                let member_fields = [MemberField::regular("m".to_string())];
                let expected = from_json(&["m"], &[texts.to_vec()], schema.clone(), &member_fields);

                let names = ["m".to_string()];
                let arrow = deserialize_arrow_chunk(
                    &names,
                    &[array.clone()],
                    schema.clone(),
                    &member_fields,
                )
                .map_err(|e| e.to_string());
                same(&expected, &arrow);

                let text = deserialize_text_chunk(
                    &names,
                    vec![read_texts.clone()],
                    schema,
                    &member_fields,
                )
                .map_err(|e| e.to_string());
                same(&expected, &text);
            }
        }

        fn all_types() -> Vec<DataType> {
            vec![
                DataType::Utf8,
                DataType::Int16,
                DataType::Int32,
                DataType::Int64,
                DataType::UInt64,
                DataType::Float32,
                DataType::Float64,
                DataType::Boolean,
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                DataType::Timestamp(TimeUnit::Millisecond, None),
                DataType::Date32,
                DataType::Decimal(10, 2),
                DataType::Decimal(38, 10),
            ]
        }

        #[test]
        fn integers() {
            let big = (1i64 << 53) + 1;
            let big_text = big.to_string();
            assert_same(
                Arc::new(Int64Array::from(vec![
                    Some(0),
                    Some(-7),
                    Some(1),
                    None,
                    Some(big),
                    Some(i64::MAX),
                ])),
                &[
                    Some("0"),
                    Some("-7"),
                    Some("1"),
                    None,
                    Some(&big_text),
                    Some("9223372036854775807"),
                ],
                &all_types(),
            );
            assert_same(
                Arc::new(Int32Array::from(vec![Some(42), None, Some(-1)])),
                &[Some("42"), None, Some("-1")],
                &all_types(),
            );
        }

        #[test]
        fn unsigned_integers() {
            assert_same(
                Arc::new(UInt64Array::from(vec![Some(u64::MAX), Some(3), None])),
                &[Some("18446744073709551615"), Some("3"), None],
                &all_types(),
            );
        }

        #[test]
        fn floats() {
            let values = [1.0, -1.5, 0.1, 1e300, 2.5, f64::INFINITY, 1e-7];
            let texts: Vec<String> = values.iter().map(|v| v.to_string()).collect();
            let mut array: Vec<Option<f64>> = values.iter().map(|v| Some(*v)).collect();
            array.push(None);
            let mut expected: Vec<Option<&str>> = texts.iter().map(|t| Some(t.as_str())).collect();
            expected.push(None);
            assert_same(Arc::new(Float64Array::from(array)), &expected, &all_types());

            let text = (1.1f32 as f64).to_string();
            assert_same(
                Arc::new(Float32Array::from(vec![Some(1.1f32), None])),
                &[Some(&text), None],
                &all_types(),
            );
        }

        #[test]
        fn booleans_and_strings() {
            assert_same(
                Arc::new(BooleanArray::from(vec![Some(true), Some(false), None])),
                &[Some("true"), Some("false"), None],
                &all_types(),
            );
            assert_same(
                Arc::new(StringArray::from(vec![
                    Some("a"),
                    Some(""),
                    None,
                    Some("2024-01-15T10:30:45.123"),
                    Some("12"),
                    Some("true"),
                ])),
                &[
                    Some("a"),
                    Some(""),
                    None,
                    Some("2024-01-15T10:30:45.123"),
                    Some("12"),
                    Some("true"),
                ],
                &all_types(),
            );
        }

        #[test]
        fn decimals() {
            let mut builder = DecimalBuilder::new(4, 18, 2);
            builder.append_value(12345).unwrap();
            builder.append_value(-5).unwrap();
            builder.append_value(100).unwrap();
            builder.append_null().unwrap();
            assert_same(
                Arc::new(builder.finish()),
                &[Some("123.45"), Some("-0.05"), Some("1"), None],
                &all_types(),
            );
        }

        #[test]
        fn timestamps() {
            // 2024-01-15T10:30:45.123456
            let micros = 1_705_314_645_123_456;
            assert_same(
                Arc::new(TimestampMicrosecondArray::from(vec![
                    Some(micros),
                    None,
                    Some(0),
                ])),
                &[
                    Some("2024-01-15T10:30:45.123"),
                    None,
                    Some("1970-01-01T00:00:00.000"),
                ],
                &all_types(),
            );
        }

        /// Columns are found by the member their alias stands for, as the
        /// rows' keys are renamed; an alias without one becomes `undefined`,
        /// a name given twice keeps its last column, a member missing from the
        /// chunk reads as NULLs, and a literal is a literal.
        #[test]
        fn names_columns_as_rows_key_them() {
            let aliases: Vec<String> = ["orders__status", "orders__count", "extra", "dup"]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let alias_to_member: HashMap<String, String> = [
                ("orders__status", "Orders.status"),
                ("orders__count", "Orders.count"),
                ("dup", "Orders.status"),
            ]
            .iter()
            .map(|(a, m)| (a.to_string(), m.to_string()))
            .collect();
            let names = stream_column_names(&aliases, Some(&alias_to_member));
            assert_eq!(
                names,
                vec![
                    "Orders.status",
                    "Orders.count",
                    "undefined",
                    "Orders.status"
                ]
            );
            assert_eq!(stream_column_names(&aliases, None), aliases);

            let columns: Vec<ArrayRef> = vec![
                Arc::new(StringArray::from(vec![Some("new"), Some("done")])),
                Arc::new(Int64Array::from(vec![Some(3), None])),
                Arc::new(StringArray::from(vec![Some("x"), Some("y")])),
                Arc::new(StringArray::from(vec![Some("last"), None])),
            ];
            let schema = schema(&[
                ("status", DataType::Utf8),
                ("count", DataType::Int64),
                ("missing", DataType::Float64),
                ("literal", DataType::Utf8),
            ]);
            let member_fields = [
                MemberField::regular("Orders.status".to_string()),
                MemberField::regular("Orders.count".to_string()),
                MemberField::regular("Orders.missing".to_string()),
                MemberField::Literal(ScalarValue::Utf8(Some("lit".to_string()))),
            ];

            // The object JS builds per row: `Orders.status` where it first
            // appears, holding the value set last.
            let expected = from_json(
                &["Orders.status", "Orders.count", "undefined"],
                &[
                    vec![Some("last"), None],
                    vec![Some("3"), None],
                    vec![Some("x"), Some("y")],
                ],
                schema.clone(),
                &member_fields,
            );
            let arrow = deserialize_arrow_chunk(&names, &columns, schema.clone(), &member_fields)
                .map_err(|e| e.to_string());
            same(&expected, &arrow);
            assert_eq!(
                arrow.unwrap().column(0).data(),
                StringArray::from(vec![Some("last"), None]).data()
            );

            let texts: Vec<Vec<Option<String>>> = columns
                .iter()
                .map(|c| {
                    arrow_text_values(c)
                        .unwrap()
                        .map(|v| match v.unwrap() {
                            FieldValue::String(s) => Some(s.into_owned()),
                            _ => None,
                        })
                        .collect()
                })
                .collect();
            let text = deserialize_text_chunk(&names, texts, schema, &member_fields)
                .map_err(|e| e.to_string());
            same(&expected, &text);
        }

        #[test]
        fn rejects_mismatched_names() {
            let schema = schema(&[("m", DataType::Utf8)]);
            let err = deserialize_arrow_chunk(
                &[],
                &[Arc::new(StringArray::from(vec!["a"])) as ArrayRef],
                schema,
                &[MemberField::regular("m".to_string())],
            )
            .unwrap_err();
            assert!(
                err.to_string().contains("1 columns, but 0 names"),
                "{}",
                err
            );
        }
    }
}
