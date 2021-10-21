use super::fetchers::Fetchers;
use super::inputs::FetchEntriesTime;
use super::utils::next_day;
use crate::wire_element::WireElement;
use ::mockall::automock;
use chrono::{Datelike, Duration, Timelike};
use hdk::prelude::*;

pub struct FetchByHourDay {}
#[cfg_attr(feature = "mock", automock)]
impl FetchByHourDay {
    pub fn fetch_entries_from_hour_to_day<
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

#[cfg(test)]
mod tests {
    use crate::crud::example::Example;
    use crate::datetime_queries::fetchers::Fetchers;
    use crate::datetime_queries::inputs::FetchEntriesTime;
    use crate::datetime_queries::{
        fetch_by_day, fetch_by_hour, fetch_entries_from_day_to_day, fetch_entries_from_day_to_hour,
        fetch_entries_from_hour_to_day, fetch_entries_from_hour_to_hour,
    };
    use crate::retrieval::get_latest_for_entry;
    use crate::wire_element::WireElement;
    use ::fixt::prelude::*;
    use hdk::prelude::*;
    #[test]
    fn test_fetch_entries_from_day_to_hour() {
        let mock_day_to_day = fetch_entries_from_day_to_day::MockFetchByDayDay::new();
        let mock_day_to_hour = fetch_entries_from_day_to_hour::MockFetchByDayHour::new();
        let mock_hour_to_day = fetch_entries_from_hour_to_day::MockFetchByHourDay::new();
        let mock_hour_to_hour = fetch_entries_from_hour_to_hour::MockFetchByHourHour::new();
        let mock_get_latest = get_latest_for_entry::MockGetLatestEntry::new();

        // should be called for 22, 23, then twice for next two days
        let start_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 20 as u32,
            hour: Some(22 as u32),
        };
        let end_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 22 as u32,
            hour: None,
        };
        let base_component = "create".to_string();
        let wire_element = WireElement::<Example> {
            header_hash: fixt![HeaderHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example { number: 1 },
        };
        let wire_vec: Vec<WireElement<Example>> = vec![wire_element.clone()];
        let wire_vec4 = vec![
            wire_element.clone(),
            wire_element.clone(),
            wire_element.clone(),
            wire_element.clone(),
        ];
        let mut mock_by_day = fetch_by_day::MockFetchByDay::new();

        // could set up the expectation each time it is expected to be called to be able to check that the inputs are as expected
        mock_by_day
            .expect_fetch_entries_by_day::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(2)
            .return_const(Ok(wire_vec.clone()));

        let mut mock_by_hour = fetch_by_hour::MockFetchByHour::new();
        mock_by_hour
            .expect_fetch_entries_by_hour::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.year),
                mockall::predicate::eq(start_time.month),
                mockall::predicate::eq(start_time.day),
                mockall::predicate::always(),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(2)
            .return_const(Ok(wire_vec.clone()));
        let mock_fetchers = Fetchers {
            day_to_day: mock_day_to_day,
            day_to_hour: mock_day_to_hour,
            hour_to_day: mock_hour_to_day,
            hour_to_hour: mock_hour_to_hour,
            day: mock_by_day,
            hour: mock_by_hour,
            get_latest: mock_get_latest,
        };
        let fetch_hour_day = super::FetchByHourDay {};
        let result = fetch_hour_day.fetch_entries_from_hour_to_day::<Example>(
            &mock_fetchers,
            start_time.clone(),
            end_time.clone(),
            base_component.clone(),
        );
        assert_eq!(result, Ok(wire_vec4.clone()));
    }
}
