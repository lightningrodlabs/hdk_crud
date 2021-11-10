use crate::chain_actions::utils::now_date_time;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Timelike, Utc};
use hdk::prelude::*;
use holo_hash::{AgentPubKey, EntryHashB64, HeaderHashB64};

#[cfg(feature = "mock")]
use ::mockall::automock;

#[derive(Debug, PartialEq, Clone)]
pub struct CreateAction {}
#[cfg_attr(feature = "mock", automock)]
impl CreateAction {
    /// This will create an entry and link it off the main Path.
    /// It can also optionally send a signal of this event (by passing `send_signal` value `true`)
    /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
    /// uses `ChainTopOrdering::Relaxed` such that multiple creates can be committed in parallel
    pub fn create_action<T, E, S>(
        &self,
        entry: T,
        path: Path,
        entry_type_id: String,
        send_signal_to_peers: Option<Vec<AgentPubKey>>,
        add_time_path: Option<String>,
    ) -> ExternResult<WireElement<T>>
    where
        Entry: 'static + TryFrom<T, Error = E>,
        WasmError: From<E>,
        T: 'static + Clone,
        AppEntryBytes: TryFrom<T, Error = E>,
        S: 'static + From<crate::signals::ActionSignal<T>> + serde::Serialize + std::fmt::Debug,
        E: 'static,
    {
        // calling create instead of create_entry to be able to indicate relaxed chain ordering
        let address = create(CreateInput::new(
            EntryDefId::App(entry_type_id.clone()),
            Entry::App(entry.clone().try_into()?),
            ChainTopOrdering::Relaxed,
        ))?;
        let entry_hash = hash_entry(entry.clone())?;
        path.ensure()?;
        let path_hash = path.hash()?;
        create_link(path_hash, entry_hash.clone(), ())?;

        match add_time_path {
            None => (),
            Some(base_component) => {
                // create a time_path
                let date: DateTime<Utc> = now_date_time()?;

                let time_path = crate::datetime_queries::utils::hour_path_from_date(
                    base_component,
                    date.year(),
                    date.month(),
                    date.day(),
                    date.hour(),
                );

                time_path.ensure()?;
                create_link(time_path.hash()?, entry_hash.clone(), ())?;
            }
        }

        let wire_entry: WireElement<T> = WireElement {
            entry,
            header_hash: HeaderHashB64::new(address),
            entry_hash: EntryHashB64::new(entry_hash),
        };

        match send_signal_to_peers {
            None => (),
            Some(vec_peers) => {
                let action_signal: crate::signals::ActionSignal<T> = crate::signals::ActionSignal {
                    entry_type: entry_type_id,
                    action: crate::signals::ActionType::Create,
                    data: crate::signals::SignalData::Create(wire_entry.clone()),
                };
                let signal = S::from(action_signal);
                let payload = ExternIO::encode(signal)?;
                remote_signal(payload, vec_peers)?;
                ()
            },
        }
        Ok(wire_entry)
    }
}
