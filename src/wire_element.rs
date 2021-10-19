use holo_hash::{EntryHashB64, HeaderHashB64};
use hdk::prelude::*;

use crate::retrieval::retrieval::EntryAndHash;

#[doc = "This data structure will be very broadly useful and represents
          how an entry should be serialized along with what metadata to
          form a consistent pattern that the UI or client can expect.
          It is called `WireElement` because it is how data looks passed
          'over the wire' or network."]
/// It serializes with camelCase style replacement of underscores in object keys.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WireElement<T> {
    pub header_hash: HeaderHashB64,
    pub entry_hash: EntryHashB64,
    pub entry: T,
}

impl<T> From<EntryAndHash<T>> for WireElement<T> {
  fn from(e: EntryAndHash<T>) -> Self {
    Self {
      entry: e.0,
      header_hash: HeaderHashB64::new(e.1),
      entry_hash: EntryHashB64::new(e.2)
    }
  }
}