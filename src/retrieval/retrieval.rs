use std::convert::identity;

use holo_hash::EntryHashB64;
use hdk::prelude::*;

use crate::wire_element::WireElement;
use crate::retrieval::get_latest_for_entry::GetLatestEntry;
/// A triple of an Entry along with the HeaderHash
/// of that committed entry and the EntryHash of the entry
pub type EntryAndHash<T> = (T, HeaderHash, EntryHash);

/// The same as an EntryAndHash but inside an Option,
/// so it can be Some(...) or None
pub type OptionEntryAndHash<T> = Option<EntryAndHash<T>>;

/// convert a SignedHeaderHashed which are like raw contents
/// into the HeaderHash of itself
pub fn get_header_hash(signed_header_hashed: element::SignedHeaderHashed) -> HeaderHash {
    signed_header_hashed.header_hashed().as_hash().to_owned()
}

/// Fetch and deserialize all the entries of a certain type that are linked to an EntryHash.
/// Useful for having a Path that you link everything to. This also internally calls [get_latest_for_entry] meaning
/// that the contents for each entry returned are automatically the latest contents.
pub fn fetch_links<
    EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
>(
    entry_hash: EntryHash,
    get_options: GetOptions,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    Ok(get_links(entry_hash, None)?
        .into_inner()
        .into_iter()
        .map(|link: link::Link| {
            let get_latest = GetLatestEntry {};
            get_latest.get_latest_for_entry::<EntryType>(link.target.clone(), get_options.clone())
        })
        .filter_map(Result::ok)
        .filter_map(identity)
        .map(|x| WireElement::from(x))
        .collect())
}

// TODO: change this in such a way that the path is only passed in if it is needed (for fetching all), for example `All(String)` pass in the path as string
/// Fetch either all entries of a certain type (assuming they are linked to a path) or a specific subset given their entry hashes.
pub fn fetch_entries<
    EntryType: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>,
>(
    entry_path: Path, // TODO: see if there is a way to derive this from the entry itself (like from entry id)
    fetch_options: FetchOptions,
    get_options: GetOptions,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    match fetch_options {
        FetchOptions::All => {
            let path_hash = entry_path.hash()?;
            fetch_links::<EntryType>(path_hash, get_options)
        }
        FetchOptions::Specific(vec_entry_hash) => {
            let entries = vec_entry_hash
                .iter()
                .map(|entry_hash| {
                    let get_latest = GetLatestEntry {}; // will probably want to pass in a struct to this function instead of this
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

#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub enum FetchOptions {
    All,
    Specific(Vec<EntryHashB64>),
}