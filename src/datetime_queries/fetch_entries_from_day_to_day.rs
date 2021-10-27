use super::fetchers::Fetchers;
use super::inputs::FetchEntriesTime;
use crate::wire_element::WireElement;
use chrono::Duration;
use hdk::prelude::*;

#[cfg(feature = "mock")]
use ::mockall::automock;

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
    fn test_fetch_entries_from_day_to_day() {
        let mock_day_to_day = fetch_entries_from_day_to_day::MockFetchByDayDay::new();
        let mock_day_to_hour = fetch_entries_from_day_to_hour::MockFetchByDayHour::new();
        let mock_hour_to_day = fetch_entries_from_hour_to_day::MockFetchByHourDay::new();
        let mock_hour_to_hour = fetch_entries_from_hour_to_hour::MockFetchByHourHour::new();
        let mock_by_hour = fetch_by_hour::MockFetchByHour::new();
        let mock_get_latest = get_latest_for_entry::MockGetLatestEntry::new();

        let start_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 20 as u32,
            hour: None,
        };
        let end_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 21 as u32,
            hour: None,
        };
        let base_component = "create".to_string();
        let wire_element = WireElement::<Example> {
            header_hash: fixt![HeaderHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example { number: 1 },
        };
        let wire_vec: Vec<WireElement<Example>> = vec![wire_element.clone()];
        let wire_vec2 = vec![wire_element.clone(), wire_element.clone()];
        let mut mock_by_day = fetch_by_day::MockFetchByDay::new();

        // fetch_entries_by_day should be called for each day in the range
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

        let mock_fetchers = Fetchers {
            day_to_day: mock_day_to_day,
            day_to_hour: mock_day_to_hour,
            hour_to_day: mock_hour_to_day,
            hour_to_hour: mock_hour_to_hour,
            day: mock_by_day,
            hour: mock_by_hour,
            get_latest: mock_get_latest,
        };
        let fetch_day_day = super::FetchByDayDay {};
        let result = fetch_day_day.fetch_entries_from_day_to_day::<Example>(
            &mock_fetchers,
            start_time,
            end_time,
            base_component.clone(),
        );
        assert_eq!(result, Ok(wire_vec2.clone()));
    }
}
