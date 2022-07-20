use super::fetchers::Fetchers;
use super::inputs::FetchEntriesTime;
use crate::wire_record::WireRecord;
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
        TY,
        E,
    >(
        &self,
        fetchers: &Fetchers,
        link_type_filter: LinkTypeFilter,
        link_type: TY,
        start: FetchEntriesTime,
        end: FetchEntriesTime,
        base_component: String,
    ) -> Result<Vec<WireRecord<EntryType>>, WasmError>
    where
        ScopedLinkType: TryFrom<TY, Error = E>,
        TY: Clone,
        WasmError: From<E>,
    {
        let mut dt = start.to_date_time();
        let mut entries = Vec::new();
        let end = end.to_date_time();
        while dt <= end {
            entries.push(fetchers.day.fetch_entries_by_day::<EntryType, TY, E>(
                &fetchers.hour,
                &fetchers.get_latest,
                link_type_filter.clone(),
                link_type.clone(),
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

    use crate::wire_record::WireRecord;
    use ::fixt::prelude::*;
    use hdk::prelude::*;
    #[test]
    fn test_fetch_entries_from_day_to_day() {
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
        let wire_vec: Vec<WireRecord<Example>> = vec![wire_record.clone()];
        let wire_vec2 = vec![wire_record.clone(), wire_record.clone()];

        let mut mock_fetchers = Fetchers::default();
        // fetch_entries_by_day should be called for each day in the range
        mock_fetchers
            .day
            .expect_fetch_entries_by_day::<Example>()
            .with(
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::always(),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(2)
            .return_const(Ok(wire_vec.clone()));

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
