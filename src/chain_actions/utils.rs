use chrono::{DateTime, NaiveDateTime, Utc, Datelike, Timelike};
use holo_hash::EntryHash;
use hdk::prelude::*;

/// get the current UTC date time
pub fn now_date_time() -> ExternResult<::chrono::DateTime<::chrono::Utc>> {
    let time = sys_time()?.as_seconds_and_nanos();

    let date: DateTime<Utc> =
        DateTime::from_utc(NaiveDateTime::from_timestamp(time.0, time.1), Utc);
    Ok(date)
}

pub fn add_current_time_path(base_component: String, entry_address: EntryHash) -> ExternResult<()>{
    let date: DateTime<Utc> = now_date_time()?;

    let time_path = crate::datetime_queries::utils::hour_path_from_date(
        base_component,
        date.year(),
        date.month(),
        date.day(),
        date.hour(),
    );

    time_path.ensure()?;
    create_link(time_path.hash()?, entry_address.clone(), ())?;
    Ok(())
}