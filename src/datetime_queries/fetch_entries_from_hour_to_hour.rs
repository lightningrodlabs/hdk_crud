use super::fetchers::Fetchers;
use super::utils::{FetchEntriesTime, next_day};
use ::mockall::automock;
use crate::retrieval::*;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use hdk::prelude::*;
use mockall_double::double;
use crate::datetime_queries::utils::{day_path_from_date, get_last_component_string, err};
use std::convert::identity;

#[double]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;

#[double]
use crate::datetime_queries::fetch_by_hour::FetchByHour;

pub struct FetchByHourHour {}
#[cfg_attr(feature = "mock", automock)]
impl FetchByHourHour {
    pub fn fetch_entries_from_hour_to_hour<
        EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
    >(
        &self,
        fetchers: &Fetchers,
        start: FetchEntriesTime,
        end: FetchEntriesTime,
        base_component: String,
    ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
        let mut dt = start.to_date_time();
        let mut entries = Vec::new();
        let end = end.to_date_time();
        let second_day = next_day(dt.clone());
        let second_last_day = end.clone() - Duration::days(1);

        // if hour range is on same day, skip first two loops
        match next_day(dt.clone()) == next_day(end.clone()) {
            true => {}
            false => {
                while dt < second_day {
                    entries.push(fetchers.hour.fetch_entries_by_hour::<EntryType>(
                        &fetchers.get_latest,
                        dt.year(),
                        dt.month(),
                        dt.day(),
                        dt.hour(),
                        base_component.clone(),
                    ));
                    dt = dt + Duration::hours(1);
                }
                while dt <= second_last_day {
                    entries.push(fetchers.day.fetch_entries_by_day::<EntryType>(
                        &fetchers.hour,
                        &fetchers.get_latest,
                        FetchEntriesTime::from_date_time(dt.clone()),
                        base_component.clone(),
                    ));
                    dt = dt + Duration::days(1);
                }
            }
        }
        while dt <= end {
            entries.push(fetchers.hour.fetch_entries_by_hour::<EntryType>(
                &fetchers.get_latest,
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                base_component.clone(),
            ));
            dt = dt + Duration::hours(1);
        }
        Ok(entries
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect())
    }
}
