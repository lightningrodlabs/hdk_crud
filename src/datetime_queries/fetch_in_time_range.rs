use hdk::prelude::*;

use super::fetchers::Fetchers;
use super::inputs::FetchEntriesTime;
use super::utils::is_valid_date_range;
use crate::wire_record::WireRecord;

/// fetches all entries of a certain type between two dates. Calls different sub methods depending on if an hour is suppled.
pub fn fetch_entries_in_time_range<
    EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
    TY,
    E,
>(
    fetchers: &Fetchers,
    link_type_filter: LinkTypeFilter,
    link_type: TY,
    start_time: FetchEntriesTime,
    end_time: FetchEntriesTime,
    base_component: String,
) -> Result<Vec<WireRecord<EntryType>>, WasmError>
where
    ScopedLinkType: TryFrom<TY, Error = E>,
    TY: Clone,
    WasmError: From<E>,
{
    is_valid_date_range(start_time.clone(), end_time.clone())?;
    match start_time.hour {
        None => {
            match end_time.hour {
                None => fetchers
                    .day_to_day
                    .fetch_entries_from_day_to_day::<EntryType, TY, E>(
                        fetchers,
                        link_type_filter,
                        link_type,
                        start_time.clone(),
                        end_time.clone(),
                        base_component,
                    ),
                Some(_) => {
                    //day to hour: loop from 1st day to 2nd last day, then loop through hours in last day
                    fetchers
                        .day_to_hour
                        .fetch_entries_from_day_to_hour::<EntryType, TY, E>(
                            fetchers,
                            link_type_filter,
                            link_type,
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
                        .fetch_entries_from_hour_to_day::<EntryType, TY, E>(
                            fetchers,
                            link_type_filter,
                            link_type,
                            start_time.clone(),
                            end_time.clone(),
                            base_component,
                        )
                }
                Some(_) => {
                    // hour to hour: loop through hours on first day, then 2nd day to 2nd last day, then hours on last day
                    fetchers
                        .hour_to_hour
                        .fetch_entries_from_hour_to_hour::<EntryType, TY, E>(
                            fetchers,
                            link_type_filter,
                            link_type,
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

    use crate::wire_record::WireRecord;
    use ::fixt::prelude::*;
    use hdk::prelude::*;

    #[test]
    fn test_fetch_in_time_range() {
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
        let wire_record = WireRecord::<Example> {
            action_hash: fixt![ActionHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example { number: 1 },
            created_at: fixt![Timestamp],
            updated_at: fixt![Timestamp],
        };
        let wire_vec: Vec<WireRecord<Example>> = vec![wire_record];
        let mut mock_fetchers = Fetchers::default();
        mock_fetchers
            .day_to_day
            .expect_fetch_entries_from_day_to_day::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.clone()),
                mockall::predicate::eq(end_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
            .return_const(Ok(wire_vec.clone()));

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
        let mut mock_fetchers = Fetchers::default();
        mock_fetchers
            .day_to_hour
            .expect_fetch_entries_from_day_to_hour::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.clone()),
                mockall::predicate::eq(end_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
            .return_const(Ok(wire_vec.clone()));

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
        let mut mock_fetchers = Fetchers::default();
        mock_fetchers
            .hour_to_day
            .expect_fetch_entries_from_hour_to_day::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.clone()),
                mockall::predicate::eq(end_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
            .return_const(Ok(wire_vec.clone()));

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
        let mut mock_fetchers = Fetchers::default();
        mock_fetchers
            .hour_to_hour
            .expect_fetch_entries_from_hour_to_hour::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(start_time.clone()),
                mockall::predicate::eq(end_time.clone()),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
            .return_const(Ok(wire_vec.clone()));

        let result = super::fetch_entries_in_time_range::<Example>(
            &mock_fetchers,
            start_time,
            end_time,
            base_component.clone(),
        );
        assert_eq!(result, Ok(wire_vec.clone()));
    }
}
