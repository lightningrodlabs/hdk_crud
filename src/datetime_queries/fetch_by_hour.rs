use crate::retrieval::*;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use hdk::prelude::*;
use mockall_double::double;
use crate::datetime_queries::original::{FetchEntriesTime, day_path_from_date, get_last_component_string, hour_path_from_date};
use std::convert::identity;
#[cfg(feature = "mock")]
use ::mockall::automock;
pub type EntryAndHash<T> = (T, HeaderHash, EntryHash);

/// The same as an EntryAndHash but inside an Option,
/// so it can be Some(...) or None
pub type OptionEntryAndHash<T> = Option<EntryAndHash<T>>;

#[double]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
pub struct FetchByHour {}
#[cfg_attr(feature = "mock", automock)]
impl FetchByHour {
    pub fn fetch_entries_by_hour<EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>>(
        &self,
        get_latest_entry: &GetLatestEntry,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        base_component: String,
    ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
        let path = hour_path_from_date(base_component.clone(), year, month, day, hour);
        let links = get_links(path.hash()?, None)?;

        let entries: Vec<WireElement<EntryType>> = links
            .into_inner()
            .into_iter()
            .map(|link| get_latest_entry.get_latest_for_entry::<EntryType>(
                link.target, 
                GetOptions::latest()
            ))
            .filter_map(Result::ok)
            .filter_map(identity)
            .map(|x| WireElement::from(x))
            .collect::<Vec<WireElement<EntryType>>>();
        Ok(entries)
    }
}

pub mod tests {
    fn test_fetch_entries_by_hour() {
        // path.hash
        // hash_entry
        // get_links
        // get_latest_for_entry
    }
}