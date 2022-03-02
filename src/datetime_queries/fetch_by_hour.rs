use crate::datetime_queries::utils::hour_path_from_date;
use crate::wire_element::WireElement;
use hdk::prelude::*;

#[cfg(feature = "mock")]
use ::mockall::automock;
use std::convert::identity;

#[cfg(not(feature = "mock"))]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
#[cfg(feature = "mock")]
use crate::retrieval::get_latest_for_entry::MockGetLatestEntry as GetLatestEntry;
pub struct FetchByHour {}
#[cfg_attr(feature = "mock", automock)]
impl FetchByHour {
    /// fetches all entries linked to a time path index for a particular hour on a specific day
    pub fn fetch_entries_by_hour<
        EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
    >(
        &self,
        get_latest_entry: &GetLatestEntry,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        base_component: String,
    ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
        let path = hour_path_from_date(base_component.clone(), year, month, day, hour);
        let links = get_links(path.path_entry_hash()?, None)?;

        let entries: Vec<WireElement<EntryType>> = links
            .into_iter()
            .map(|link| {
                get_latest_entry
                    .get_latest_for_entry::<EntryType>(link.target, GetOptions::latest())
            })
            .filter_map(Result::ok)
            .filter_map(identity)
            .map(|x| WireElement::from(x))
            .collect::<Vec<WireElement<EntryType>>>();
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use crate::crud::example::Example;
    use crate::retrieval::get_latest_for_entry;
    use crate::wire_element::WireElement;
    use ::fixt::prelude::*;
    use hdk::prelude::*;

    #[test]
    fn test_fetch_entries_by_hour() {
        let mut mock_hdk = MockHdkT::new();

        // set up for the first expected hash_entry call
        let path = Path::from("create.2021-10-15.10");
        let path_hash = fixt!(EntryHash);
        let path_entry = PathEntry::new(path_hash.clone());
        let path_entry_hash = fixt!(EntryHash);
        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(Entry::try_from(path.clone()).unwrap()))
            .times(1)
            .return_const(Ok(path_hash.clone()));

        mock_hdk
            .expect_hash_entry()
            .with(mockall::predicate::eq(Entry::try_from(path_entry.clone()).unwrap()))
            .times(1)
            .return_const(Ok(path_entry_hash.clone()));

        let get_links_input = vec![GetLinksInput::new(path_entry_hash, None)];

        // creating an expected output of get_links, which is a Vec<Links>, and Links is a Vec<Link>
        // let link_tag: LinkTag = LinkTag::try_from(&Path::from("create.2021-10-15.10")).unwrap();
        let bytes: Vec<u8> = "10".try_into().unwrap();
        // let link_tag: LinkTag = LinkTag::try_from(SerializedBytes::from(UnsafeBytes::from("10"))).unwrap();
        let link_tag: LinkTag = LinkTag::new(bytes);

        let link_output = Link {
            target: fixt![EntryHash],
            timestamp: fixt![Timestamp],
            tag: link_tag,
            create_link_hash: fixt![HeaderHash],
        };

        let get_links_output = vec![vec![link_output.clone()]];

        mock_hdk
            .expect_get_links()
            .with(mockall::predicate::eq(get_links_input))
            .times(1)
            .return_const(Ok(get_links_output));

        let get_latest_output = Some(WireElement::<Example> {
            header_hash: fixt![HeaderHashB64],
            entry_hash: fixt![EntryHashB64],
            entry: Example { number: 1 },
            created_at: fixt![Timestamp],
            updated_at: fixt![Timestamp],
        });

        // set up a mock of get_latest_for_entry
        let mut mock_get_latest = get_latest_for_entry::MockGetLatestEntry::new();
        mock_get_latest
            .expect_get_latest_for_entry::<Example>()
            .with(
                mockall::predicate::eq(link_output.target),
                mockall::predicate::eq(GetOptions::latest()),
            )
            .times(1)
            .return_const(Ok(get_latest_output.clone()));

        set_hdk(mock_hdk);
        let base_component = "create".to_string();
        let fetch_by_hour = super::FetchByHour {};
        let result = fetch_by_hour.fetch_entries_by_hour::<Example>(
            &mock_get_latest,
            2021,
            10 as u32,
            15 as u32,
            10 as u32,
            base_component,
        );
        let output = vec![WireElement::from(get_latest_output.unwrap())];
        assert_eq!(result, Ok(output));
    }
}
