use hdk::prelude::*;

use crate::retrieval::retrieval::*;

#[cfg(feature = "mock")]
use ::mockall::automock;
/// A triple of an Entry along with the HeaderHash
/// of that committed entry and the EntryHash of the entry
pub type EntryAndHash<T> = (T, HeaderHash, EntryHash);

/// The same as an EntryAndHash but inside an Option,
/// so it can be Some(...) or None
pub type OptionEntryAndHash<T> = Option<EntryAndHash<T>>;

#[derive(Debug, PartialEq, Clone)]
pub struct GetLatestEntry {}
#[cfg_attr(feature = "mock", automock)]
impl GetLatestEntry {
    /// If an entry at the `entry_hash` has multiple updates to itself, this
    /// function will sort through them by timestamp in order to return the contents
    /// of the latest update. It also has the special behaviour of returning the
    /// ORIGINAL HeaderHash, as opposed to the HeaderHash of the Header that performed
    /// that latest update. This is useful if you want hashes in your application
    /// to act consistently, almost acting as an "id" in a centralized system.
    /// It simplifies traversal of the update tree, since all updates
    /// made by the client can reference the original, instead of updates reference updates
    pub fn get_latest_for_entry<T: 'static + TryFrom<SerializedBytes, Error = SerializedBytesError>>(
        &self,
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
}
