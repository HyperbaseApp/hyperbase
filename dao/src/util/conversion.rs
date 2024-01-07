use anyhow::{Error, Result};
use chrono::{DateTime, NaiveDate, NaiveTime, Timelike, Utc};
use scylla::frame::value::{
    CqlDate as ScyllaCqlDate, CqlTime as ScyllaCqlTime, CqlTimestamp as ScyllaCqlTimestamp,
};

pub fn scylla_cql_timestamp_to_datetime_utc(
    timestamp: &ScyllaCqlTimestamp,
) -> Result<DateTime<Utc>> {
    let milliseconds_since_epoch = timestamp.0; // in ms
    let secs = milliseconds_since_epoch / 10_i64.pow(3);
    let nsecs: u32 =
        u32::try_from((milliseconds_since_epoch - secs * 10_i64.pow(3)) * 10_i64.pow(6))?;
    Ok(DateTime::from_timestamp(secs, nsecs).ok_or_else(||Error::msg("Can't convert value with type 'timestamp' from ScyllaDB to 'datetime'. Value is out of range."))?)
}

pub fn scylla_cql_time_to_naivetime(time: &ScyllaCqlTime) -> Result<NaiveTime> {
    let nanoseconds_since_midnight = time.0;
    let secs = nanoseconds_since_midnight / 10_i64.pow(9);
    let nano = nanoseconds_since_midnight - (secs * 10_i64.pow(9));
    Ok(
        NaiveTime::from_num_seconds_from_midnight_opt(u32::try_from(secs)?, u32::try_from(nano)?)
            .ok_or_else(|| {
            Error::msg(
            "Can't convert value with type 'time' from ScyllaDB to 'time'. Value is out of range.",
        )
        })?,
    )
}

pub fn naivetime_to_scylla_cql_time(time: &NaiveTime) -> Result<ScyllaCqlTime> {
    Ok(ScyllaCqlTime(
        i64::from(time.num_seconds_from_midnight()) * 10_i64.pow(9),
    ))
}

pub fn scylla_cql_date_to_naivedate(date: &ScyllaCqlDate) -> Result<NaiveDate> {
    Ok(NaiveDate::from_yo_opt(1970, 1)
        .unwrap()
        .checked_add_signed(chrono::Duration::days(date.0 as i64 - (1 << 31)))
        .ok_or_else(|| {
            Error::msg(
            "Can't convert value with type 'date' from ScyllaDB to 'date'. Value is out of range.",
        )
        })?)
}

pub fn naivedate_to_scylla_cql_date(date: &NaiveDate) -> Result<ScyllaCqlDate> {
    Ok(ScyllaCqlDate(u32::try_from(
        (1 << 31)
            + date
                .signed_duration_since(NaiveDate::from_yo_opt(1970, 1).unwrap())
                .num_days(),
    )?))
}
