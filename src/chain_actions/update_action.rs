use hdk::prelude::*;
use holo_hash::{AgentPubKey, EntryHashB64, HeaderHashB64};
use crate::wire_element::WireElement;

/// This will add an update to an entry.
/// It can also optionally send a signal of this event (by passing `send_signal` value `true`)
/// to all peers returned by the `get_peers` call given during the macro call to `crud!`
/// uses `ChainTopOrdering::Relaxed` such that multiple updates can be committed in parallel
pub fn update_action<T, E, S>(
    entry: T,
    header_hash: HeaderHashB64,
    path_string: String,
    send_signal: bool,
    convert_to_receiver_signal: fn(crate::signals::ActionSignal<T>) -> S,
    get_peers: fn() -> ExternResult<Vec<AgentPubKey>>,
) -> ExternResult<WireElement<T>>
where
    Entry: TryFrom<T, Error = E>,
    WasmError: From<E>,
    T: Clone,
    AppEntryBytes: TryFrom<T, Error = E>,
    S: serde::Serialize + std::fmt::Debug,
{
    // calling update instead of update_entry to be able to indicate relaxed chain ordering
    hdk::entry::update(
        header_hash.clone().into(),
        CreateInput::new(
            EntryDefId::App(path_string.clone()),
            Entry::App(entry.clone().try_into()?),
            ChainTopOrdering::Relaxed,
        ),
    )?;
    let entry_address = hash_entry(entry.clone())?;
    let wire_entry: WireElement<T> = WireElement {
        entry,
        header_hash,
        entry_hash: EntryHashB64::new(entry_address),
    };
    if send_signal {
        let action_signal: crate::signals::ActionSignal<T> = crate::signals::ActionSignal {
            entry_type: path_string,
            action: crate::signals::ActionType::Update,
            data: crate::signals::SignalData::Update(wire_entry.clone()),
        };
        let signal = convert_to_receiver_signal(action_signal);
        let payload = ExternIO::encode(signal)?;
        let peers = get_peers()?;
        remote_signal(payload, peers)?;
    }
    Ok(wire_entry)
}
