#[cfg_attr(feature = "mock", double)]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
use crate::wire_element::WireElement;
use hdk::prelude::*;
#[cfg(feature = "mock")]
use mockall_double::double;
use std::convert::identity;

#[cfg(feature = "mock")]
use ::mockall::automock;

#[derive(Debug, PartialEq, Clone)]
pub struct FetchLinks {}
#[cfg_attr(feature = "mock", automock)]
impl FetchLinks {
    pub fn new() -> Self {
        Self {}
    }
    /// Fetch and deserialize all the entries of a certain type that are linked to an EntryHash.
    /// Useful for having a Path that you link everything to. This also internally calls [get_latest_for_entry] meaning
    /// that the contents for each entry returned are automatically the latest contents.
    pub fn fetch_links<
        EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
    >(
        &self,
        get_latest: &GetLatestEntry,
        entry_hash: EntryHash,
        get_options: GetOptions,
    ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
        Ok(get_links(entry_hash, None)?
            .into_inner()
            .into_iter()
            .map(|link: link::Link| {
                get_latest
                    .get_latest_for_entry::<EntryType>(link.target.clone(), get_options.clone())
            })
            .filter_map(Result::ok)
            .filter_map(identity)
            .map(|x| WireElement::from(x))
            .collect())
    }
}
