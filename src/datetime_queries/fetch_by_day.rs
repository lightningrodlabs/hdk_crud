use crate::retrieval::*;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use hdk::prelude::*;
use mockall_double::double;
use crate::datetime_queries::original::{FetchEntriesTime, day_path_from_date, get_last_component_string, err};
use std::convert::identity;

#[double]
use crate::datetime_queries::fetch_by_hour::FetchByHour;

pub struct FetchByDay {}
impl FetchByDay {
    pub fn fetch_entries_by_day<EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>>(
        fetch_by_hour: &FetchByHour,
        time: FetchEntriesTime,
        base_component: String,
    ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
        let path = day_path_from_date(base_component.clone(), time.year, time.month, time.day);
        let children = path.children()?;

        let entries = children
            .into_inner()
            .into_iter()
            .map(|hour_link| {
                let hour_str = get_last_component_string(hour_link.tag)?;

                let hour = hour_str.parse::<u32>().or(Err(err("Invalid path")))?;

                fetch_by_hour.fetch_entries_by_hour::<EntryType>(
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
    use crate::datetime_queries::original::FetchEntriesTime;
    use crate::wire_element::WireElement;
    use crate::crud::example::Example;
    use assert_matches::assert_matches;

    #[test]
    fn test_fetch_entries_by_day() {
        let mut mock_hdk = MockHdkT::new();

        let path = Path::from("create.2021-10-15");
        let path_entry = Entry::try_from(path.clone()).unwrap();
        let path_hash = fixt!(EntryHash);
        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(path_entry.clone()))
            .times(1)
            .return_const(Ok(path_hash.clone()));

        // set up io for get
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

        // set up input and outputs for hash entry
        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(path_entry.clone()))
            .times(1)
            .return_const(Ok(path_hash.clone()));

        // set up input for get links
        // pub const NAME: [u8; 8] = [0x68, 0x64, 0x6b, 0x2e, 0x70, 0x61, 0x74, 0x68];
        // let name = NAME;
        let get_links_input = vec![GetLinksInput::new(
            path_hash,
            Some(holochain_zome_types::link::LinkTag::new(NAME)),
        )];
        // Links is a vec of Link, how does the base `get_links` structure the list of links? within one element of vec<Links> or one link per Links?
        // constructing this vec of links would be important to testing the functionality
        
        // add descriptive comments here
        let link_tag: LinkTag = LinkTag::try_from(&Path::from("create.2021-10-15.10")).unwrap();

        let link_output = Link {
            target: fixt![EntryHash],
            timestamp: fixt![Timestamp],
            tag: link_tag,
            create_link_hash: fixt![HeaderHash],
        };

        let get_links_output = vec![Links::from(vec![link_output])];
        
        mock_hdk
            .expect_get_links()
            .with(mockall::predicate::eq(get_links_input))
            .times(1)
            .return_const(Ok(get_links_output));

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

        mock_queries
            .expect_fetch_entries_by_hour::<Example>()
            .with(
                mockall::predicate::eq(fetch_time.year),
                mockall::predicate::eq(fetch_time.month),
                mockall::predicate::eq(fetch_time.day),
                mockall::predicate::eq(10 as u32),
                mockall::predicate::eq(base_component.clone()),
            )
            .times(1)
            .return_const(Ok(hour_entries));


        set_hdk(mock_hdk);
        
        let result = super::FetchByDay::fetch_entries_by_day::<Example>(
            &mock_queries, 
            fetch_time, 
            base_component
        );
        assert_matches!(result, Ok(hour_entries));
    }
}