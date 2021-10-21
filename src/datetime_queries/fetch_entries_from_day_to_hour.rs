use crate::wire_element::WireElement;
use chrono::{Datelike, Duration, Timelike};
use hdk::prelude::*;
use crate::datetime_queries::utils::FetchEntriesTime;
use ::mockall::automock;
use super::fetchers::Fetchers;

pub struct FetchByDayHour {}
#[cfg_attr(feature = "mock", automock)]
impl FetchByDayHour {
    pub fn fetch_entries_from_day_to_hour<
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
        let end_prev = end - Duration::days(1); // this is to prevent fetch entries by day being called on the last day (we don't want all the hours on the last day)
        while dt < end_prev {
            entries.push(fetchers.day.fetch_entries_by_day::<EntryType>(
                &fetchers.hour,
                &fetchers.get_latest,
                FetchEntriesTime::from_date_time(dt.clone()),
                base_component.clone(),
            ));
            dt = dt + Duration::days(1);
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

#[cfg(test)]
mod tests {
    use crate::crud::example::Example;
    use crate::retrieval::get_latest_for_entry;
    use crate::wire_element::WireElement;
    use ::fixt::prelude::*;
    use hdk::prelude::*;
    use crate::datetime_queries::fetchers::Fetchers;
    use crate::datetime_queries::{fetch_entries_from_day_to_day, fetch_entries_from_hour_to_day, fetch_entries_from_day_to_hour, fetch_entries_from_hour_to_hour, fetch_by_day, fetch_by_hour};
    use crate::datetime_queries::utils::FetchEntriesTime;
    
    #[test]
    fn test_fetch_entries_from_day_to_hour(){
        let mock_day_to_day = fetch_entries_from_day_to_day:: MockFetchByDayDay::new();
        let mock_day_to_hour = fetch_entries_from_day_to_hour::MockFetchByDayHour::new();
        let mock_hour_to_day = fetch_entries_from_hour_to_day::MockFetchByHourDay::new();
        let mock_hour_to_hour = fetch_entries_from_hour_to_hour::MockFetchByHourHour::new();
        let mock_get_latest = get_latest_for_entry::MockGetLatestEntry::new();
       
        // case 1: fetch by day is called once, and fetch by hour is called 3 times
        // another case to try out would be same start and end day, so only fetch by hour is called
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
            hour: Some(2 as u32),
        };
        let base_component = "create".to_string();
        let wire_element = WireElement::<Example> {
            header_hash: fixt![HeaderHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example { number: 1 },
        };
        let wire_vec: Vec<WireElement<Example>> = vec![wire_element.clone()];
        let wire_vec4 = vec![wire_element.clone(), wire_element.clone(), wire_element.clone(), wire_element.clone()];
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
            .times(1)
            .return_const(Ok(wire_vec.clone()));
        
        let mut mock_by_hour = fetch_by_hour::MockFetchByHour::new();
        mock_by_hour
            .expect_fetch_entries_by_hour::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.year),
                mockall::predicate::eq(start_time.month),
                mockall::predicate::eq(end_time.day),
                mockall::predicate::always(),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(3)
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
        let fetch_day_hour = super::FetchByDayHour {};
        let result = fetch_day_hour.fetch_entries_from_day_to_hour::<Example>(
            &mock_fetchers, 
            start_time.clone(), 
            end_time.clone(), 
            base_component.clone()
        );
        assert_eq!(result, Ok(wire_vec4.clone()));
    }
}
