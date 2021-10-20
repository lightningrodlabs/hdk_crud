use crate::retrieval::*;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use hdk::prelude::*;
use mockall_double::double;

use std::convert::identity;
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
use crate::datetime_queries::{fetch_by_day::FetchByDay, fetch_by_hour::FetchByHour};


// pub fn fetch_entries_by_time<EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
//     time: FetchEntriesTime,
//     base_component: String,
// ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
//     Ok(match time.hour {
//         None => fetch_entries_by_day(time, base_component),
//         Some(h) => fetch_entries_by_hour(time.year, time.month, time.day, h, base_component),
//     }?)
// }

// pub fn fetch_entries_in_time_range<
//     EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>,
// >(
//     start_time: FetchEntriesTime,
//     end_time: FetchEntriesTime,
//     base_component: String,
// ) -> Result<Vec<WireElement<hdk::prelude::EntryType>>, hdk::prelude::WasmError> {
//     is_valid_date_range(start_time.clone(), end_time.clone())?;
//     match start_time.hour {
//         None => {
//             match end_time.hour {
//                 None => fetch_entries_from_day_to_day(
//                     start_time.clone(),
//                     end_time.clone(),
//                     base_component,
//                 ),
//                 Some(_) => {
//                     //day to hour: loop from 1st day to 2nd last day, then loop through hours in last day
//                     fetch_entries_from_day_to_hour(
//                         start_time.clone(),
//                         end_time.clone(),
//                         base_component,
//                     )
//                 }
//             }
//         }
//         Some(_) => {
//             match end_time.hour {
//                 None => {
//                     // hour to day: loop through hours on first day, then 2nd day to last day
//                     fetch_entries_from_hour_to_day(
//                         start_time.clone(),
//                         end_time.clone(),
//                         base_component,
//                     )
//                 }
//                 Some(_) => {
//                     // hour to hour: loop through hours on first day, then 2nd day to 2nd last day, then hours on last day
//                     fetch_entries_from_hour_to_hour(
//                         start_time.clone(),
//                         end_time.clone(),
//                         base_component,
//                     )
//                 }
//             }
//         }
//     }
// }
// fn fetch_entries_from_day_to_day(
//     start: FetchEntriesTime,
//     end: FetchEntriesTime,
//     base_component: String,
// ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
//     let mut dt = start.to_date_time();
//     let mut entries = Vec::new();
//     let end = end.to_date_time();
//     while dt <= end {
//         entries.push(fetch_entries_by_day::<EntryType>(
//             FetchEntriesTime::from_date_time(dt.clone()),
//             base_component.clone(),
//         ));
//         dt = dt + Duration::days(1);
//     }
//     Ok(entries
//         .into_iter()
//         .filter_map(Result::ok)
//         .flatten()
//         .collect())
// }

// fn fetch_entries_from_day_to_hour(
//     start: FetchEntriesTime,
//     end: FetchEntriesTime,
//     base_component: String,
// ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
//     let mut dt = start.to_date_time();
//     let mut entries = Vec::new();
//     let end = end.to_date_time();
//     while dt < end {
//         entries.push(fetch_entries_by_day::<EntryType>(
//             FetchEntriesTime::from_date_time(dt.clone()),
//             base_component.clone(),
//         ));
//         dt = dt + Duration::days(1);
//     }
//     while dt <= end {
//         entries.push(fetch_entries_by_hour::<EntryType>(
//             dt.year(),
//             dt.month(),
//             dt.day(),
//             dt.hour(),
//             base_component.clone(),
//         ));
//         dt = dt + Duration::hours(1);
//     }
//     Ok(entries
//         .into_iter()
//         .filter_map(Result::ok)
//         .flatten()
//         .collect())
// }

// fn fetch_entries_from_hour_to_day(
//     start: FetchEntriesTime,
//     end: FetchEntriesTime,
//     base_component: String,
// ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
//     let mut dt = start.to_date_time();
//     let mut entries = Vec::new();
//     let end = end.to_date_time();
//     let second_day = next_day(dt.clone());
//     while dt < second_day {
//         entries.push(fetch_entries_by_hour::<EntryType>(
//             dt.year(),
//             dt.month(),
//             dt.day(),
//             dt.hour(),
//             base_component.clone(),
//         ));
//         dt = dt + Duration::hours(1);
//     }
//     while dt <= end {
//         entries.push(fetch_entries_by_day::<EntryType>(
//             FetchEntriesTime::from_date_time(dt.clone()),
//             base_component.clone(),
//         ));
//         dt = dt + Duration::days(1);
//     }
//     Ok(entries
//         .into_iter()
//         .filter_map(Result::ok)
//         .flatten()
//         .collect())
// }

// fn fetch_entries_from_hour_to_hour(
//     start: FetchEntriesTime,
//     end: FetchEntriesTime,
//     base_component: String,
// ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
//     let mut dt = start.to_date_time();
//     let mut entries = Vec::new();
//     let end = end.to_date_time();
//     let second_day = next_day(dt.clone());
//     let second_last_day = end.clone() - Duration::days(1);

//     // if hour range is on same day, skip first two loops
//     match next_day(dt.clone()) == next_day(end.clone()) {
//         true => {}
//         false => {
//             while dt < second_day {
//                 entries.push(fetch_entries_by_hour::<EntryType>(
//                     dt.year(),
//                     dt.month(),
//                     dt.day(),
//                     dt.hour(),
//                     base_component.clone(),
//                 ));
//                 dt = dt + Duration::hours(1);
//             }
//             while dt <= second_last_day {
//                 entries.push(fetch_entries_by_day::<EntryType>(
//                     FetchEntriesTime::from_date_time(dt.clone()),
//                     base_component.clone(),
//                 ));
//                 dt = dt + Duration::days(1);
//             }
//         }
//     }
//     while dt <= end {
//         entries.push(fetch_entries_by_hour::<EntryType>(
//             dt.year(),
//             dt.month(),
//             dt.day(),
//             dt.hour(),
//             base_component.clone(),
//         ));
//         dt = dt + Duration::hours(1);
//     }
//     Ok(entries
//         .into_iter()
//         .filter_map(Result::ok)
//         .flatten()
//         .collect())
// }

fn is_valid_date_range(start: FetchEntriesTime, end: FetchEntriesTime) -> Result<(), WasmError> {
    match start.to_date_time() < end.to_date_time() {
        true => Ok(()),
        false => Err(err("invalid date range")),
    }
}
fn next_day(date_time: DateTime<Utc>) -> DateTime<Utc> {
    let next_day = date_time + Duration::days(1);
    DateTime::from_utc(
        NaiveDate::from_ymd(next_day.year(), next_day.month(), next_day.day()).and_hms(0, 0, 0),
        Utc,
    )
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FetchEntriesTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: Option<u32>,
}

impl FetchEntriesTime {
    fn to_date_time(&self) -> DateTime<Utc> {
        match self.hour {
            None => DateTime::from_utc(
                NaiveDate::from_ymd(self.year, self.month, self.day).and_hms(0, 0, 0),
                Utc,
            ),
            Some(h) => DateTime::from_utc(
                NaiveDate::from_ymd(self.year, self.month, self.day).and_hms(h, 0, 0),
                Utc,
            ),
        }
    }
    fn from_date_time(dt: DateTime<Utc>) -> Self {
        Self {
            year: dt.year(),
            month: dt.month(),
            day: dt.day(),
            hour: Some(dt.hour()),
        }
    }
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

