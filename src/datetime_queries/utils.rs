use super::inputs::FetchEntriesTime;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use hdk::prelude::*;

pub fn is_valid_date_range(
    start: FetchEntriesTime,
    end: FetchEntriesTime,
) -> Result<(), WasmError> {
    match start.to_date_time() < end.to_date_time() {
        true => Ok(()),
        false => Err(err("invalid date range")),
    }
}
pub fn next_day(date_time: DateTime<Utc>) -> DateTime<Utc> {
    let next_day = date_time + Duration::days(1);
    DateTime::from_utc(
        NaiveDate::from_ymd(next_day.year(), next_day.month(), next_day.day()).and_hms(0, 0, 0),
        Utc,
    )
}

pub fn err(reason: &str) -> WasmError {
    WasmError::Guest(String::from(reason))
}

pub fn get_last_component_string(path_tag: LinkTag) -> ExternResult<String> {
    let hour_path = Path::try_from(&path_tag)?;
    let hour_components: Vec<hdk::hash_path::path::Component> = hour_path.into();

    let hour_bytes: &hdk::hash_path::path::Component =
        hour_components.last().ok_or(err("Invalid path"))?;
    let hour_str: String = hour_bytes.try_into()?;

    Ok(hour_str)
}

pub fn day_path_from_date(base_component: String, year: i32, month: u32, day: u32) -> Path {
    Path::from(format!("{}.{}-{}-{}", base_component, year, month, day))
}

pub fn hour_path_from_date(
    base_component: String,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
) -> Path {
    Path::from(format!(
        "{}.{}-{}-{}.{}",
        base_component, year, month, day, hour
    ))
}
