use datafusion::arrow::{datatypes::Schema, ipc::writer::StreamWriter, record_batch::RecordBatch};

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
    let mut writer = StreamWriter::try_new(Vec::new(), &batch.schema())?;
    writer.write(batch)?;
    writer.finish()?;
    Ok(writer.into_inner()?)
}
