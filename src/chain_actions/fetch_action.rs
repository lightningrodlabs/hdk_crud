use hdk::prelude::*;
use crate::wire_element::WireElement;

/// This is the exposed/public Zome function for either fetching ALL or a SPECIFIC list of the entries of the type.
pub fn fetch_action<T, E>(
    fetch_options: crate::retrieval::retrieval::FetchOptions,
    get_options: GetOptions,
    path: Path,
) -> ExternResult<Vec<WireElement<T>>>
where
    Entry: TryFrom<T, Error = E>,
    WasmError: From<E>,
    T: 'static + Clone + TryFrom<SerializedBytes, Error = SerializedBytesError>,
{
    let get_latest = crate::retrieval::get_latest_for_entry::GetLatestEntry {};
    let entries = crate::retrieval::retrieval::fetch_entries::<T>( // TODO: change to struct method
        &get_latest,
        path,
        fetch_options,
        get_options,
    )?;
    Ok(entries)
}
