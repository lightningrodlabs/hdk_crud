#[double]
use crate::datetime_queries::fetch_by_day::FetchByDay;
#[double]
use crate::datetime_queries::fetch_by_hour::FetchByHour;
use crate::datetime_queries::utils::{
    day_path_from_date, get_last_component_string, hour_path_from_date, FetchEntriesTime,
};
#[double]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
use crate::retrieval::*;
use crate::wire_element::WireElement;
#[cfg(feature = "mock")]
use ::mockall::automock;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use hdk::prelude::*;
use mockall_double::double;
use std::convert::identity;

pub struct TimeFetcher {
    fetch_by_day: FetchByDay,
    fetch_by_hour: FetchByHour,
}

pub fn fetch_entries_by_time<
    EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
>(
    fetch_by_day: &FetchByDay,
    fetch_by_hour: &FetchByHour,
    get_latest_entry: &GetLatestEntry,
    time: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    Ok(match time.hour {
        None => fetch_by_day.fetch_entries_by_day(
            &fetch_by_hour,
            &get_latest_entry,
            time,
            base_component,
        ),
        Some(h) => fetch_by_hour.fetch_entries_by_hour(
            &get_latest_entry,
            time.year,
            time.month,
            time.day,
            h,
            base_component,
        ),
    }?)
}

#[cfg(test)]
mod tests {
    use crate::crud::example::Example;
    use crate::datetime_queries::utils::FetchEntriesTime;
    use crate::datetime_queries::{fetch_by_day, fetch_by_hour};
    use crate::retrieval::get_latest_for_entry;
    use crate::wire_element::WireElement;
    use ::fixt::prelude::*;
    use assert_matches::assert_matches;
    use hdk::hash_path::path::NAME;
    use hdk::prelude::*;
    use holochain_types::prelude::{ElementFixturator, LinkTagFixturator};

    #[test]
    fn test_fetch_by_time_day() {
        // when calling fetch_entries_by_time without
        // an 'hour' then verify that it calls
        // fetch_entries_by_day

        let fetch_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 15 as u32,
            hour: None,
        };

        let base_component = "create".to_string();
        let wire_element = WireElement::<Example> {
            header_hash: fixt![HeaderHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example { number: 1 },
        };
        let wire_vec: Vec<WireElement<Example>> = vec![wire_element];
        let mut mock_fetch_by_day = fetch_by_day::MockFetchByDay::new();

        mock_fetch_by_day
            .expect_fetch_entries_by_day::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(fetch_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
            .return_const(Ok(wire_vec.clone()));

        let mut fetch_by_hour = fetch_by_hour::MockFetchByHour::new();
        let mut get_latest_entry = get_latest_for_entry::MockGetLatestEntry::new();
        let result = super::fetch_entries_by_time::<Example>(
            &mock_fetch_by_day,
            &fetch_by_hour,
            &get_latest_entry,
            fetch_time,
            base_component,
        );
        assert_eq!(result, Ok(wire_vec));
    }
    #[test]
    fn test_fetch_by_time_hour() {
        // when calling fetch_entries_by_time with
        // an 'hour' then verify that it calls
        // fetch_entries_by_hour

        let fetch_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 15 as u32,
            hour: Some(10 as u32),
        };

        let base_component = "create".to_string();
        let wire_element = WireElement::<Example> {
            header_hash: fixt![HeaderHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example { number: 1 },
        };
        let wire_vec: Vec<WireElement<Example>> = vec![wire_element];
        let mut mock_fetch_by_day = fetch_by_day::MockFetchByDay::new();

        let mut mock_fetch_by_hour = fetch_by_hour::MockFetchByHour::new();
        mock_fetch_by_hour
            .expect_fetch_entries_by_hour::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(fetch_time.year),
                mockall::predicate::eq(fetch_time.month),
                mockall::predicate::eq(fetch_time.day),
                mockall::predicate::eq(fetch_time.hour.unwrap()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
            .return_const(Ok(wire_vec.clone()));
        let mut get_latest_entry = get_latest_for_entry::MockGetLatestEntry::new();
        let result = super::fetch_entries_by_time::<Example>(
            &mock_fetch_by_day,
            &mock_fetch_by_hour,
            &get_latest_entry,
            fetch_time,
            base_component,
        );
        assert_eq!(result, Ok(wire_vec));
    }
}
