#[cfg(not(feature = "mock"))]
use crate::retrieval::fetch_entries::FetchEntries;
#[cfg(not(feature = "mock"))]
use crate::retrieval::fetch_links::FetchLinks;
#[cfg(not(feature = "mock"))]
use crate::retrieval::get_latest_for_entry::GetLatestEntry;

#[cfg(feature = "mock")]
use crate::retrieval::fetch_entries::MockFetchEntries as FetchEntries;
#[cfg(feature = "mock")]
use crate::retrieval::fetch_links::MockFetchLinks as FetchLinks;
#[cfg(feature = "mock")]
use crate::retrieval::get_latest_for_entry::MockGetLatestEntry as GetLatestEntry;

use crate::wire_element::WireElement;
use hdk::prelude::*;

#[cfg(feature = "mock")]
use ::mockall::automock;

#[derive(Debug, PartialEq, Clone)]
pub struct FetchAction {}
#[cfg_attr(feature = "mock", automock)]
impl FetchAction {
    /// This is the exposed/public Zome function for either fetching ALL or a SPECIFIC list of the entries of the type.
    pub fn fetch_action<T, E>(
        &self,
        fetch_entries: &FetchEntries,
        fetch_links: &FetchLinks,
        get_latest: &GetLatestEntry,
        fetch_options: crate::retrieval::inputs::FetchOptions,
        get_options: GetOptions,
        path: Path,
    ) -> ExternResult<Vec<WireElement<T>>>
    where
        Entry: TryFrom<T, Error = E>,
        WasmError: From<E>,
        T: 'static + Clone + TryFrom<SerializedBytes, Error = SerializedBytesError>,
        E: 'static,
    {
        let entries = fetch_entries.fetch_entries::<T>(
            fetch_links,
            &get_latest,
            path,
            fetch_options,
            get_options,
        )?;
        Ok(entries)
    }
}
