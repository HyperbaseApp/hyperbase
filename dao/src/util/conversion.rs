use anyhow::{Error, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};

pub fn datetime_to_duration_since_epoch<T: TimeZone>(datetime: &DateTime<T>) -> Duration {
    Duration::milliseconds(datetime.timestamp_millis())
}

pub fn duration_since_epoch_to_datetime(duration: &Duration) -> Result<DateTime<Utc>> {
    match Utc.timestamp_millis_opt(duration.num_milliseconds()) {
        chrono::LocalResult::None => Err(Error::msg("Failed to convert duration to datetime")),
        chrono::LocalResult::Single(datetime) => Ok(datetime),
        chrono::LocalResult::Ambiguous(_, _) => Err(Error::msg(
            "Failed to convert duration to datetime because it is ambiguous",
        )),
    }
}
