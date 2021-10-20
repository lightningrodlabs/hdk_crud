use crate::retrieval::*;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use hdk::prelude::*;
use mockall_double::double;

use crate::datetime_queries::{fetch_by_day::FetchByDay, fetch_by_hour::FetchByHour};
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
use std::convert::identity;
use ::mockall::automock;
use super::fetchers::Fetchers;
use super::utils::FetchEntriesTime;
#[derive(Clone)]
pub struct FetchByDayDay {}
#[cfg_attr(feature = "mock", automock)]
impl FetchByDayDay {
    pub fn fetch_entries_from_day_to_day<
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
        while dt <= end {
            entries.push(fetchers.day.fetch_entries_by_day::<EntryType>(
                &fetchers.hour,
                &fetchers.get_latest,
                FetchEntriesTime::from_date_time(dt.clone()),
                base_component.clone(),
            ));
            dt = dt + Duration::days(1);
        }
        Ok(entries
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect())
    }
}
