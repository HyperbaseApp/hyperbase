use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use scylla::frame::value::CqlTimestamp as ScyllaCqlTimestamp;

pub fn scylla_cql_timestamp_to_datetime_utc(
    timestamp: &ScyllaCqlTimestamp,
) -> Result<DateTime<Utc>> {
    let timestamp = timestamp.0; // in ms
    let secs = timestamp / 10_i64.pow(3);
    let nsecs = u32::try_from((timestamp * 10_i64.pow(6)) - (secs * 10_i64.pow(9)))?;
    match DateTime::from_timestamp(secs, nsecs) {
        Some(timestamp) => Ok(timestamp),
        None => return Err(Error::msg("Failed to get timestamp")),
    }
}
