use chrono::{DateTime, NaiveDateTime, Utc};
use hdk::prelude::ExternResult;
use hdk::time::sys_time;

/// get the current UTC date time
pub fn now_date_time() -> ExternResult<::chrono::DateTime<::chrono::Utc>> {
    let time = sys_time()?.as_seconds_and_nanos();

    let date: DateTime<Utc> =
        DateTime::from_utc(NaiveDateTime::from_timestamp(time.0, time.1), Utc);
    Ok(date)
}
