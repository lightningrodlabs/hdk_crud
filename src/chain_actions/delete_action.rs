use hdk::prelude::*;
use holo_hash::{AgentPubKey, HeaderHashB64};

#[cfg(feature = "mock")]
use ::mockall::automock;

/// a struct which implements a [delete_action](DeleteAction::delete_action) method
/// a method is used instead of a function so that it can be mocked to simplify unit testing
#[derive(Debug, PartialEq, Clone)]
pub struct DeleteAction {}
#[cfg_attr(feature = "mock", automock)]
impl DeleteAction {
    /// This will mark the entry at `address` as "deleted".
    /// It can also optionally send a signal of this event to all peers supplied in `send_signal_to_peers`
    /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
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
        delete_entry(DeleteInput::new(header_hash.clone().into(), ChainTopOrdering::Relaxed))?;
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
            }
        }
        Ok(header_hash)
    }
}
