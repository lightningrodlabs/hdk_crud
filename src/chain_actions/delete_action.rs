use hdk::prelude::*;
use holo_hash::{AgentPubKey, HeaderHashB64};

/// This will mark the entry at `address` as "deleted".
/// It can also optionally send a signal of this event (by passing `send_signal` value `true`)
/// to all peers returned by the `get_peers` call given during the macro call to `crud!`
#[doc = "This will be called with `send_signal` as `true` by [delete_[entry_type]]"]
pub fn delete_action<T, E, S>(
    header_hash: HeaderHashB64,
    path_string: String,
    send_signal: bool,
    convert_to_receiver_signal: fn(crate::signals::ActionSignal<T>) -> S,
    get_peers: fn() -> ExternResult<Vec<AgentPubKey>>,
) -> ExternResult<HeaderHashB64>
where
    Entry: TryFrom<T, Error = E>,
    WasmError: From<E>,
    T: Clone,
    AppEntryBytes: TryFrom<T, Error = E>,
    S: serde::Serialize + std::fmt::Debug,
{
    delete_entry(header_hash.clone().into())?;
    if send_signal {
        let action_signal: crate::signals::ActionSignal<T> = crate::signals::ActionSignal {
            entry_type: path_string,
            action: crate::signals::ActionType::Delete,
            data: crate::signals::SignalData::Delete::<T>(header_hash.clone()),
        };
        let signal = convert_to_receiver_signal(action_signal);
        let payload = ExternIO::encode(signal)?;
        let peers = get_peers()?;
        remote_signal(payload, peers)?;
    }
    Ok(header_hash)
}