use crate::datetime_queries::inputs::FetchEntriesTime;
use crate::datetime_queries::utils::{day_path_from_date, err, get_last_component_string};
use crate::wire_record::WireRecord;
use hdk::prelude::*;

#[cfg(feature = "mock")]
use ::mockall::automock;

#[cfg(not(feature = "mock"))]
use crate::datetime_queries::fetch_by_hour::FetchByHour;
#[cfg(not(feature = "mock"))]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;

#[cfg(feature = "mock")]
use crate::datetime_queries::fetch_by_hour::MockFetchByHour as FetchByHour;
#[cfg(feature = "mock")]
use crate::retrieval::get_latest_for_entry::MockGetLatestEntry as GetLatestEntry;
#[derive(Clone)]
pub struct FetchByDay {}
#[cfg_attr(feature = "mock", automock)]
impl FetchByDay {
    /// fetches all entries linked to a time path index for a certain day
    pub fn fetch_entries_by_day<
        EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
        TY,
        E,
    >(
        &self,
        fetch_by_hour: &FetchByHour,
        get_latest_entry: &GetLatestEntry,
        link_type_filter: LinkTypeFilter,
        link_type: TY,
        time: FetchEntriesTime,
        base_component: String,
    ) -> Result<Vec<WireRecord<EntryType>>, WasmError>
    where
        ScopedLinkType: TryFrom<TY, Error = E>,
        TY: Clone,
        WasmError: From<E>,
    {
        let path = day_path_from_date(
            link_type.clone(),
            base_component.clone(),
            time.year,
            time.month,
            time.day,
        )?;
        // TODO: wrap in path.exists which would add extra hdk calls to be mocked in the test
        let children = path.children()?;
        let entries = children
            .into_iter()
            .map(|hour_link| {
                let hour_str = get_last_component_string(hour_link.tag)?;
                let hour = hour_str.parse::<u32>().or(Err(err("Invalid path")))?;
                fetch_by_hour.fetch_entries_by_hour::<EntryType, TY, E>(
                    &get_latest_entry,
                    link_type_filter.clone(),
                    link_type.clone(),
                    time.year,
                    time.month,
                    time.day,
                    hour,
                    base_component.clone(),
                )
            })
            .filter_map(Result::ok)
            .flatten()
            .collect();
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use crate::crud::example::Example;
    use crate::datetime_queries::fetch_by_hour;
    use crate::datetime_queries::inputs::FetchEntriesTime;
    use crate::retrieval::get_latest_for_entry;
    use crate::wire_record::WireRecord;
    use ::fixt::prelude::*;
    use hdk::hash_path::path::{Component, DHT_PREFIX};
    use hdk::prelude::*;
    use holochain_types::prelude::RecordFixturator;

    #[test]
    fn test_fetch_entries_by_day() {
        let mut mock_hdk = MockHdkT::new();

        // when fetch_entries_by_day calls path.children(), assuming the path already exists, the following hdk
        // functions are called: hash_entry x4, get, get_links

        // set up for the first expected hash_entry call

        let path = Path::from("create.2021-10-15");
        let path_hash = fixt!(EntryHash);
        let path_entry = PathEntry::new(path_hash.clone());
        let path_entry_hash = fixt!(EntryHash);
        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(
                Entry::try_from(path.clone()).unwrap(),
            ))
            .times(1)
            .return_const(Ok(path_hash.clone()));

        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(
                Entry::try_from(path_entry.clone()).unwrap(),
            ))
            .times(1)
            .return_const(Ok(path_entry_hash.clone()));
        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(
                Entry::try_from(path.clone()).unwrap(),
            ))
            .times(1)
            .return_const(Ok(path_hash.clone()));

        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(
                Entry::try_from(path_entry.clone()).unwrap(),
            ))
            .times(1)
            .return_const(Ok(path_entry_hash.clone()));

        // // set up for expected get call
        let path_get_input = vec![GetInput::new(
            AnyDhtHash::from(path_entry_hash.clone()),
            GetOptions::local(),
        )];
        let expected_get_output = vec![Some(fixt!(Record))]; // this should return the path
        mock_hdk
            .expect_get()
            .with(mockall::predicate::eq(path_get_input))
            .times(1)
            .return_const(Ok(expected_get_output));

        // set up input for get links, the second parameter is the default used by the Holochain code
        let get_links_input = vec![GetLinksInput::new(
            path_entry_hash,
            Some(holochain_zome_types::link::LinkTag::new([DHT_PREFIX])),
        )];

        // creating an expected output of get_links, which is a Vec<Links>, and Links is a Vec<Link>
        // since the link tag is used to get the hour component from the path, it must be constructed properly
        let hour_component = Component::from(String::from("10"));
        let link_tag = LinkTag::new(
            [DHT_PREFIX]
                .iter()
                .chain(
                    <Vec<u8>>::from(UnsafeBytes::from(
                        SerializedBytes::try_from(hour_component).unwrap(),
                    ))
                    .iter(),
                )
                .cloned()
                .collect::<Vec<u8>>(),
        );

        let link_output = Link {
            target: fixt![EntryHash],
            timestamp: fixt![Timestamp],
            tag: link_tag,
            create_link_hash: fixt![ActionHash],
        };

        // here we are assuming there is only one hour component to the day path, however if we wanted to
        // make sure the code properly cycles through each hour component, we would add extra Link records
        // to the below vector
        let get_links_output = vec![vec![link_output]];

        mock_hdk
            .expect_get_links()
            .with(mockall::predicate::eq(get_links_input))
            .times(1)
            .return_const(Ok(get_links_output));

        // initializing expected inputs and outputs for the mocked fetch_entries_by_hour
        let fetch_time = FetchEntriesTime {
            year: 2021,
            month: 10 as u32,
            day: 15 as u32,
            hour: None,
        };
        let base_component = "create".to_string();
        let hour_entry = WireRecord::<Example> {
            action_hash: fixt![ActionHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example { number: 1 },
            created_at: fixt![Timestamp],
            updated_at: fixt![Timestamp],
        };
        let hour_entries: Vec<WireRecord<Example>> = vec![hour_entry];
        // set up a mock of fetch_entries_by_hour
        let mut mock_queries = fetch_by_hour::MockFetchByHour::new();
        let mock_latest_entry = get_latest_for_entry::MockGetLatestEntry::new();
        mock_queries
            .expect_fetch_entries_by_hour::<Example>()
            .with(
                // MockGetLatestEntry does not implement PartialEq so can't be compared
                mockall::predicate::always(),
                mockall::predicate::eq(fetch_time.year),
                mockall::predicate::eq(fetch_time.month),
                mockall::predicate::eq(fetch_time.day),
                mockall::predicate::eq(10 as u32),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
            .return_const(Ok(hour_entries.clone()));

        set_hdk(mock_hdk);
        let fetch_by_day = super::FetchByDay {};
        let result = fetch_by_day.fetch_entries_by_day::<Example>(
            &mock_queries,
            &mock_latest_entry,
            fetch_time,
            base_component,
        );
        assert_eq!(result, Ok(hour_entries));
    }
}
