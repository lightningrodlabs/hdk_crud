use crate::retrieval::*;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use hdk::prelude::*;
use mockall_double::double;
use crate::datetime_queries::utils::{FetchEntriesTime, day_path_from_date, get_last_component_string, err};
use std::convert::identity;
use ::mockall::automock;
#[double]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;

#[double]
use crate::datetime_queries::fetch_by_hour::FetchByHour;

#[derive(Clone)]
pub struct FetchByDay {}
#[cfg_attr(feature = "mock", automock)]
impl FetchByDay {
    pub fn fetch_entries_by_day<EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>>(
        &self,
        fetch_by_hour: &FetchByHour,
        get_latest_entry: &GetLatestEntry,
        time: FetchEntriesTime,
        base_component: String,
    ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
        let path = day_path_from_date(base_component.clone(), time.year, time.month, time.day);
        // TODO: wrap in path.exists
        let children = path.children()?;

        let entries = children
            .into_inner()
            .into_iter()
            .map(|hour_link| {
                let hour_str = get_last_component_string(hour_link.tag)?;

                let hour = hour_str.parse::<u32>().or(Err(err("Invalid path")))?;
                fetch_by_hour.fetch_entries_by_hour::<EntryType>(
                    &get_latest_entry,
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
    use ::fixt::prelude::*;
    use hdk::hash_path::path::NAME;
    use holochain_types::prelude::{ElementFixturator, LinkTagFixturator};
    use hdk::prelude::*;
    use crate::datetime_queries::fetch_by_hour;
    use crate::datetime_queries::utils::FetchEntriesTime;
    use crate::wire_element::WireElement;
    use crate::crud::example::Example;
    use crate::retrieval::get_latest_for_entry;
    use assert_matches::assert_matches;

    #[test]
    fn test_fetch_entries_by_day() {
        let mut mock_hdk = MockHdkT::new();

        // when fetch_entries_by_day calls path.children(), assuming the path already exists, the following hdk
        // functions are called: hash_entry, get, hash_entry, get_links

        // set up for the first expected hash_entry call
        let path = Path::from("create.2021-10-15");
        let path_entry = Entry::try_from(path.clone()).unwrap();
        let path_hash = fixt!(EntryHash);
        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(path_entry.clone()))
            .times(1)
            .return_const(Ok(path_hash.clone()));

        // set up for expected get call
        let path_get_input = vec![GetInput::new(
            AnyDhtHash::from(path_hash.clone()),
            GetOptions::content(),
        )];
        let expected_get_output = vec![Some(fixt!(Element))]; // this should return the path
        mock_hdk
            .expect_get()
            .with(mockall::predicate::eq(path_get_input))
            .times(1)
            .return_const(Ok(expected_get_output));

        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(path_entry.clone()))
            .times(1)
            .return_const(Ok(path_hash.clone()));

        // set up input for get links, the second parameter is the default used by the Holochain code
        let get_links_input = vec![GetLinksInput::new(
            path_hash,
            Some(holochain_zome_types::link::LinkTag::new(NAME)),
        )];
        
        // creating an expected output of get_links, which is a Vec<Links>, and Links is a Vec<Link>
        // since the link tag is used to get the hour component from the path, it must be constructed properly
        let link_tag: LinkTag = LinkTag::try_from(&Path::from("create.2021-10-15.10")).unwrap();

        let link_output = Link {
            target: fixt![EntryHash],
            timestamp: fixt![Timestamp],
            tag: link_tag,
            create_link_hash: fixt![HeaderHash],
        };

        // here we are assuming there is only one hour component to the day path, however if we wanted to 
        // make sure the code properly cycles through each hour component, we would add extra Link elements
        // to the below vector
        let get_links_output = vec![Links::from(vec![link_output])];
        
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
        let hour_entry = WireElement::<Example> {
            header_hash: fixt![HeaderHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example {number: 1},
        };
        let hour_entries: Vec<WireElement<Example>> = vec![hour_entry];
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
            base_component
        );
        assert_eq!(result, Ok(hour_entries));
    }
}