use chrono::{DateTime, Duration, TimeZone};

pub fn datetime_to_duration_since_epoch<T: TimeZone>(datetime: DateTime<T>) -> Duration {
    Duration::milliseconds(datetime.timestamp_millis())
}
