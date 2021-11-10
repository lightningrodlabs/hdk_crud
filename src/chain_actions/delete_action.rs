use hdk::prelude::*;
use holo_hash::{AgentPubKey, HeaderHashB64};

#[cfg(feature = "mock")]
use ::mockall::automock;

#[derive(Debug, PartialEq, Clone)]
pub struct DeleteAction {}
#[cfg_attr(feature = "mock", automock)]
impl DeleteAction {
    /// This will mark the entry at `address` as "deleted".
    /// It can also optionally send a signal of this event (by passing `send_signal` value `true`)
    /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
    // #[doc = "This will be called with `send_signal` as `true` by [delete_[entry_type]]"]
    pub fn delete_action<T, E, S>(
        &self,
        header_hash: HeaderHashB64,
        entry_type_id: String,
        send_signal_to_peers: Option<Vec<AgentPubKey>>,
    ) -> ExternResult<HeaderHashB64>
    where
        Entry: 'static + TryFrom<T, Error = E>,
        WasmError: 'static + From<E>,
        T: 'static + Clone,
        AppEntryBytes: 'static + TryFrom<T, Error = E>,
        S: 'static + From<crate::signals::ActionSignal<T>> + serde::Serialize + std::fmt::Debug,
        E: 'static,
    {
        delete_entry(header_hash.clone().into())?;
        match send_signal_to_peers {
            None => (),
            Some(vec_peers) => {
                let action_signal: crate::signals::ActionSignal<T> = crate::signals::ActionSignal {
                    entry_type: entry_type_id,
                    action: crate::signals::ActionType::Delete,
                    data: crate::signals::SignalData::Delete::<T>(header_hash.clone()),
                };
                let signal = S::from(action_signal);
                let payload = ExternIO::encode(signal)?;
                remote_signal(payload, vec_peers)?;
            },
        }
        Ok(header_hash)
    }
}
