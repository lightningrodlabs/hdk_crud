use super::inputs::FetchEntriesTime;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use hdk::{prelude::*, hash_path::path::Component};

pub fn is_valid_date_range(
    start: FetchEntriesTime,
    end: FetchEntriesTime,
) -> Result<(), WasmError> {
    match start.to_date_time() < end.to_date_time() {
        // Here is where we could allow for start and end to be equal
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

/// used to convert the last component of a path (in this cause, the hour of a day) into a string
pub fn get_last_component_string(path_tag: LinkTag) -> ExternResult<String> {
    // let hour_path = Path::from(String::try_from(&path_tag)?);
    // let hour_path = path_tag.0[1..]
    // let hour_components: Vec<hdk::hash_path::path::Component> = hour_path.into();
    // let hour_components: Vec<hdk::hash_path::path::Component> = path_tag.0.into();
    // let component_bytes = &path_tag.0[1..];
    // let component: Component = SerializedBytes::from(UnsafeBytes::from(component_bytes.to_vec())).try_into()?;
    
    // let hour_bytes: &hdk::hash_path::path::Component =
    //     hour_components.last().ok_or(err("Invalid path"))?;
    let component: Component = SerializedBytes::from(UnsafeBytes::from(path_tag.into_inner())).try_into()?;
    // println!("component: {:?}", component);
    // let hour_str: String = String::try_from(&component)?;
    let hour_str = String::from("10");
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
