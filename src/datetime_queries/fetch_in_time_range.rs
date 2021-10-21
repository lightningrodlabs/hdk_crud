use hdk::prelude::*;

use super::fetchers::Fetchers;
use super::utils::is_valid_date_range;
use super::inputs::FetchEntriesTime;
use crate::wire_element::WireElement;

pub fn fetch_entries_in_time_range<
    EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
>(
    fetchers: &Fetchers,
    start_time: FetchEntriesTime,
    end_time: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    is_valid_date_range(start_time.clone(), end_time.clone())?;
    match start_time.hour {
        None => {
            match end_time.hour {
                None => fetchers
                    .day_to_day
                    .fetch_entries_from_day_to_day::<EntryType>(
                        fetchers,
                        start_time.clone(),
                        end_time.clone(),
                        base_component,
                    ),
                Some(_) => {
                    //day to hour: loop from 1st day to 2nd last day, then loop through hours in last day
                    fetchers
                        .day_to_hour
                        .fetch_entries_from_day_to_hour::<EntryType>(
                            fetchers,
                            start_time.clone(),
                            end_time.clone(),
                            base_component,
                        )
                }
            }
        }
        Some(_) => {
            match end_time.hour {
                None => {
                    // hour to day: loop through hours on first day, then 2nd day to last day
                    fetchers
                        .hour_to_day
                        .fetch_entries_from_hour_to_day::<EntryType>(
                            fetchers,
                            start_time.clone(),
                            end_time.clone(),
                            base_component,
                        )
                }
                Some(_) => {
                    // hour to hour: loop through hours on first day, then 2nd day to 2nd last day, then hours on last day
                    fetchers
                        .hour_to_hour
                        .fetch_entries_from_hour_to_hour::<EntryType>(
                            fetchers,
                            start_time.clone(),
                            end_time.clone(),
                            base_component,
                        )
                }
            }
        }
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
    fn test_fetch_in_time_range() {
        let mock_day_to_hour = fetch_entries_from_day_to_hour::MockFetchByDayHour::new();
        let mock_hour_to_day = fetch_entries_from_hour_to_day::MockFetchByHourDay::new();
        let mock_hour_to_hour = fetch_entries_from_hour_to_hour::MockFetchByHourHour::new();
        let mock_by_day = fetch_by_day::MockFetchByDay::new();
        let mock_by_hour = fetch_by_hour::MockFetchByHour::new();
        let mock_get_latest = get_latest_for_entry::MockGetLatestEntry::new();

        // ==================================================
        // day to day test
        // ==================================================
        // create inputs and outputs
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
        let wire_vec: Vec<WireElement<Example>> = vec![wire_element];
        let mut mock_day_to_day = fetch_entries_from_day_to_day::MockFetchByDayDay::new();
        mock_day_to_day
            .expect_fetch_entries_from_day_to_day::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.clone()),
                mockall::predicate::eq(end_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
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

        let result = super::fetch_entries_in_time_range::<Example>(
            &mock_fetchers,
            start_time,
            end_time,
            base_component.clone(),
        );
        assert_eq!(result, Ok(wire_vec.clone()));
        // ==================================================
        // day to hour test
        // ==================================================

        let mock_day_to_day = fetch_entries_from_day_to_day::MockFetchByDayDay::new();
        let mock_hour_to_day = fetch_entries_from_hour_to_day::MockFetchByHourDay::new();
        let mock_hour_to_hour = fetch_entries_from_hour_to_hour::MockFetchByHourHour::new();
        let mock_by_day = fetch_by_day::MockFetchByDay::new();
        let mock_by_hour = fetch_by_hour::MockFetchByHour::new();
        let mock_get_latest = get_latest_for_entry::MockGetLatestEntry::new();
        // create inputs and outputs
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
            hour: Some(10 as u32),
        };
        let mut mock_day_to_hour = fetch_entries_from_day_to_hour::MockFetchByDayHour::new();
        mock_day_to_hour
            .expect_fetch_entries_from_day_to_hour::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.clone()),
                mockall::predicate::eq(end_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
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

        let result = super::fetch_entries_in_time_range::<Example>(
            &mock_fetchers,
            start_time,
            end_time,
            base_component.clone(),
        );
        assert_eq!(result, Ok(wire_vec.clone()));
        // ==================================================
        // hour to day test
        // ==================================================

        let mock_day_to_day = fetch_entries_from_day_to_day::MockFetchByDayDay::new();
        let mock_day_to_hour = fetch_entries_from_day_to_hour::MockFetchByDayHour::new();
        let mock_hour_to_hour = fetch_entries_from_hour_to_hour::MockFetchByHourHour::new();
        let mock_by_day = fetch_by_day::MockFetchByDay::new();
        let mock_by_hour = fetch_by_hour::MockFetchByHour::new();
        let mock_get_latest = get_latest_for_entry::MockGetLatestEntry::new();
        // create inputs and outputs
        let start_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 20 as u32,
            hour: Some(10 as u32),
        };
        let end_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 21 as u32,
            hour: None,
        };
        let mut mock_hour_to_day = fetch_entries_from_hour_to_day::MockFetchByHourDay::new();
        mock_hour_to_day
            .expect_fetch_entries_from_hour_to_day::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.clone()),
                mockall::predicate::eq(end_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
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

        let result = super::fetch_entries_in_time_range::<Example>(
            &mock_fetchers,
            start_time,
            end_time,
            base_component.clone(),
        );
        assert_eq!(result, Ok(wire_vec.clone()));
        // ==================================================
        // hour to day test
        // ==================================================

        let mock_day_to_day = fetch_entries_from_day_to_day::MockFetchByDayDay::new();
        let mock_day_to_hour = fetch_entries_from_day_to_hour::MockFetchByDayHour::new();
        let mock_hour_to_day = fetch_entries_from_hour_to_day::MockFetchByHourDay::new();
        let mock_by_day = fetch_by_day::MockFetchByDay::new();
        let mock_by_hour = fetch_by_hour::MockFetchByHour::new();
        let mock_get_latest = get_latest_for_entry::MockGetLatestEntry::new();
        // create inputs and outputs
        let start_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 20 as u32,
            hour: Some(10 as u32),
        };
        let end_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 21 as u32,
            hour: Some(10 as u32),
        };
        let mut mock_hour_to_hour = fetch_entries_from_hour_to_hour::MockFetchByHourHour::new();
        mock_hour_to_hour
            .expect_fetch_entries_from_hour_to_hour::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.clone()),
                mockall::predicate::eq(end_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
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

        let result = super::fetch_entries_in_time_range::<Example>(
            &mock_fetchers,
            start_time,
            end_time,
            base_component.clone(),
        );
        assert_eq!(result, Ok(wire_vec.clone()));
    }
}
