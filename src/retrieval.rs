use std::convert::identity;

use holo_hash::EntryHashB64;
use hdk::prelude::*;

use crate::wire_element::WireElement;

/// A triple of an Entry along with the HeaderHash
/// of that committed entry and the EntryHash of the entry
pub type EntryAndHash<T> = (T, HeaderHash, EntryHash);

/// The same as an EntryAndHash but inside an Option,
/// so it can be Some(...) or None
pub type OptionEntryAndHash<T> = Option<EntryAndHash<T>>;

/// convert a SignedHeaderHashed which are like raw contents
/// into the HeaderHash of itself
fn get_header_hash(signed_header_hashed: element::SignedHeaderHashed) -> HeaderHash {
    signed_header_hashed.header_hashed().as_hash().to_owned()
}

/// If an entry at the `entry_hash` has multiple updates to itself, this
/// function will sort through them by timestamp in order to return the contents
/// of the latest update. It also has the special behaviour of returning the
/// ORIGINAL HeaderHash, as opposed to the HeaderHash of the Header that performed
/// that latest update. This is useful if you want hashes in your application
/// to act consistently, almost acting as an "id" in a centralized system.
/// It simplifies traversal of the update tree, since all updates
/// made by the client can reference the original, instead of updates reference updates
pub fn get_latest_for_entry<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    entry_hash: EntryHash,
    get_options: GetOptions,
) -> ExternResult<OptionEntryAndHash<T>> {
    // First, make sure we DO have the latest header_hash address
    let maybe_latest_header_hash = match get_details(entry_hash.clone(), get_options.clone())? {
        Some(Details::Entry(details)) => match details.entry_dht_status {
            metadata::EntryDhtStatus::Live => match details.updates.len() {
                // pass out the header associated with this entry
                0 => Some(get_header_hash(details.headers.first().unwrap().to_owned())),
                _ => {
                    let mut sortlist = details.updates.to_vec();
                    // unix timestamp should work for sorting
                    sortlist.sort_by_key(|update| update.header().timestamp().as_millis());
                    // sorts in ascending order, so take the last element
                    let last = sortlist.last().unwrap().to_owned();
                    Some(get_header_hash(last))
                }
            },
            metadata::EntryDhtStatus::Dead => None,
            _ => None,
        },
        _ => None,
    };

    // Second, go and get that element, and return it and its header_address
    match maybe_latest_header_hash {
        Some(latest_header_hash) => match get(latest_header_hash, get_options)? {
            Some(element) => match element.entry().to_app_option::<T>()? {
                Some(entry) => Ok(Some((
                    entry,
                    match element.header() {
                        // we DO want to return the header for the original
                        // instead of the updated, in our case
                        Header::Update(update) => update.original_header_address.clone(),
                        Header::Create(_) => element.header_address().clone(),
                        _ => unreachable!("Can't have returned a header for a nonexistent entry"),
                    },
                    element.header().entry_hash().unwrap().to_owned(),
                ))),
                None => Ok(None),
            },
            None => Ok(None),
        },
        None => Ok(None),
    }
}

/// Fetch and deserialize all the entries of a certain type that are linked to an EntryHash.
/// Useful for having a Path that you link everything to. This also internally calls [get_latest_for_entry] meaning
/// that the contents for each entry returned are automatically the latest contents.
pub fn fetch_links<
    EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>,
>(
    entry_hash: EntryHash,
    get_options: GetOptions,
) -> Result<Vec<WireElement<EntryType>>, WasmError> {
    Ok(get_links(entry_hash, None)?
        .into_inner()
        .into_iter()
        .map(|link: link::Link| {
            get_latest_for_entry::<EntryType>(link.target.clone(), get_options.clone())
        })
        .filter_map(Result::ok)
        .filter_map(identity)
        .map(|x| WireElement::from(x))
        .collect())
}

/// Fetch either all entries of a certain type (assuming they are linked to a path) or a specific subset given their entry hashes.
pub fn fetch_entries<
    EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>,
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
                    get_latest_for_entry::<EntryType>(
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FetchEntriesTime {
    pub year: usize,
    pub month: usize,
    pub day: usize,
    pub hour: Option<usize>,
}