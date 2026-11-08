use datafusion::arrow::{
    array::{make_array, ArrayRef, MutableArrayData},
    datatypes::Schema,
    ipc::writer::StreamWriter,
    record_batch::RecordBatch,
};

use crate::CubeError;

/// Encodes `schema` as an Arrow IPC stream holding no batches.
pub fn encode_schema(schema: &Schema) -> Result<Vec<u8>, CubeError> {
    let mut writer = StreamWriter::try_new(Vec::new(), schema)?;
    writer.finish()?;
    Ok(writer.into_inner()?)
}

/// Encodes `batch` as a self-contained Arrow IPC stream: its schema, then the
/// batch, so a client can decode each batch on its own.
pub fn encode_batch(batch: &RecordBatch) -> Result<Vec<u8>, CubeError> {
    let batch = compact(batch)?;
    let mut writer = StreamWriter::try_new(Vec::new(), &batch.schema())?;
    writer.write(&batch)?;
    writer.finish()?;
    Ok(writer.into_inner()?)
}

/// `batch` with every column copied down to its own rows. A batch sliced from
/// a larger one shares the larger one's buffers, and the IPC writer writes
/// buffers whole, so each slice of a result would carry all of it.
fn compact(batch: &RecordBatch) -> Result<RecordBatch, CubeError> {
    if batch.num_columns() == 0 {
        return Ok(batch.clone());
    }

    let columns: Vec<ArrayRef> = batch
        .columns()
        .iter()
        .map(|column| {
            let mut copy = MutableArrayData::new(vec![column.data()], false, column.len());
            copy.extend(0, 0, column.len());
            make_array(copy.freeze())
        })
        .collect();

    Ok(RecordBatch::try_new(batch.schema(), columns)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::{
        array::{Float64Array, StringArray},
        datatypes::{DataType, Field},
        ipc::reader::StreamReader,
    };
    use std::{io::Cursor, sync::Arc};

    fn decode(bytes: Vec<u8>) -> Vec<RecordBatch> {
        StreamReader::try_new(Cursor::new(bytes), None)
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn test_slices_are_encoded_with_their_own_rows_only() {
        let rows = 100_000;
        let strings: StringArray = (0..rows)
            .map(|i| (i % 7 != 0).then(|| format!("value-{:010}", i)))
            .collect();
        let floats: Float64Array = (0..rows).map(|i| Some(i as f64)).collect();
        let schema = Arc::new(Schema::new(vec![
            Field::new("s", DataType::Utf8, true),
            Field::new("f", DataType::Float64, true),
        ]));
        let full = RecordBatch::try_new(schema, vec![Arc::new(strings), Arc::new(floats)]).unwrap();

        let whole = encode_batch(&full).unwrap().len();
        for offset in [0, 50_003] {
            let slice = full.slice(offset, 10);
            let bytes = encode_batch(&slice).unwrap();
            assert!(
                bytes.len() < whole / 100,
                "a 10-row slice took {} bytes, the whole batch {}",
                bytes.len(),
                whole
            );
            assert_eq!(decode(bytes), vec![slice]);
        }
    }

    #[test]
    fn test_batches_without_columns_encode() {
        let batch = RecordBatch::new_empty(Arc::new(Schema::empty()));
        assert!(encode_batch(&batch).is_ok());
    }
}
