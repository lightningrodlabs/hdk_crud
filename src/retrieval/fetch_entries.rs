use hdk::prelude::*;
use std::convert::identity;
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
use crate::wire_element::WireElement;
use crate::retrieval::inputs::FetchOptions; 
use crate::retrieval::fetch_links::FetchLinks;
#[cfg(feature = "mock")]
use ::mockall::automock;

#[derive(Debug, PartialEq, Clone)]
pub struct FetchEntries {}
#[cfg_attr(feature = "mock", automock)]
impl FetchEntries {
    // TODO: change this in such a way that the path is only passed in if it is needed (for fetching all), for example `All(String)` pass in the path as string
    /// Fetch either all entries of a certain type (assuming they are linked to a path) or a specific subset given their entry hashes.
    pub fn fetch_entries<
        EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
    >(
        &self,
        fetch_links: &FetchLinks,
        get_latest: &GetLatestEntry,
        entry_path: Path, // TODO: see if there is a way to derive this from the entry itself (like from entry id)
        fetch_options: FetchOptions,
        get_options: GetOptions,
    ) -> Result<Vec<WireElement<EntryType>>, WasmError> {
        match fetch_options {
            FetchOptions::All => {
                let path_hash = entry_path.hash()?;
                fetch_links.fetch_links::<EntryType>(get_latest, path_hash, get_options) // TODO: will have to instantiate or pass in the struct
            }
            FetchOptions::Specific(vec_entry_hash) => {
                let entries = vec_entry_hash
                    .iter()
                    .map(|entry_hash| {
                        get_latest.get_latest_for_entry::<EntryType>(
                            entry_hash.clone().into(),
                            get_options.clone(),
                        )
                    })
                    // drop Err(_) and unwraps Ok(_)
                    .filter_map(Result::ok)
                    // drop None and unwraps Some(_)
                    .filter_map(identity)
                    .map(|x| WireElement::from(x))
                    .collect();
                Ok(entries)
            }
        }
    }
}