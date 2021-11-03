use hdk::prelude::*;
use std::convert::identity;
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
use crate::wire_element::WireElement;

#[cfg(feature = "mock")]
use ::mockall::automock;

#[derive(Debug, PartialEq, Clone)]
pub struct FetchLinks {}
#[cfg_attr(feature = "mock", automock)]
impl FetchLinks {
    pub fn fetch_links<
        EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>
    >(
        get_latest: &GetLatestEntry,
        entry_hash: EntryHash,
        get_options: GetOptions,
    ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
        Ok(get_links(entry_hash, None)?
            .into_inner()
            .into_iter()
            .map(|link: link::Link| {
                get_latest.get_latest_for_entry::<EntryType>(link.target.clone(), get_options.clone())
            })
            .filter_map(Result::ok)
            .filter_map(identity)
            .map(|x| WireElement::from(x))
            .collect()
        )
    }
}