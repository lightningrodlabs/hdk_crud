use hdk::prelude::*;

/// A triple of an Entry along with the ActionHash
/// of that committed entry and the EntryHash of the entry
pub type EntryAndHash<T> = (T, ActionHash, EntryHash);

/// The same as an EntryAndHash but inside an Option,
/// so it can be Some(...) or None
pub type OptionEntryAndHash<T> = Option<EntryAndHash<T>>;

/// convert a SignedActionHashed which are like raw contents
/// into the ActionHash of itself
pub fn get_action_hash(signed_action_hashed: record::SignedActionHashed) -> ActionHash {
    signed_action_hashed.as_hash().to_owned()
}
