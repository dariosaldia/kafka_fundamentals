use rdkafka::producer::FutureRecord;

use crate::config::PartitioningMode;

#[derive(Debug, thiserror::Error)]
pub enum BuildRecordError {
    #[error("a key is required for keyed partitioning")]
    MissingKey,
}

pub fn create_future_record<'a>(
    maybe_key: Option<&'a str>,
    payload: &'a [u8],
    topic: &'a str,
    partition_mode: PartitioningMode,
) -> Result<FutureRecord<'a, str, [u8]>, BuildRecordError> {
    let base = FutureRecord::to(topic).payload(payload);
    let record = match partition_mode {
        PartitioningMode::Keyed => {
            let k = maybe_key.ok_or(BuildRecordError::MissingKey)?;
            base.key(k)
        }
        PartitioningMode::RoundRobin => base,
    };
    Ok(record)
}
