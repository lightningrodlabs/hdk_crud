use hdk::prelude::*;
use chrono::{DateTime, Datelike, Timelike, Utc};
use holo_hash::{AgentPubKey, EntryHashB64, HeaderHashB64};
use crate::wire_element::WireElement;
use crate::crud::now_date_time;

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
        entry: T,
        path: Path,
        path_string: String,
        send_signal: bool,
        add_time_path: Option<String>,
        // convert_to_receiver_signal: fn(crate::signals::ActionSignal<T>) -> S,
        // get_peers: fn() -> ExternResult<Vec<AgentPubKey>>,
    ) -> ExternResult<WireElement<T>>
    where
        Entry: 'static + TryFrom<T, Error = E>,
        WasmError: From<E>,
        T: 'static + Clone,
        AppEntryBytes: TryFrom<T, Error = E>,
        S: 'static + serde::Serialize + std::fmt::Debug,
        E: 'static, 
    {
        // calling create instead of create_entry to be able to indicate relaxed chain ordering
        let address = create(CreateInput::new(
            EntryDefId::App(path_string.clone()),
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

        // if send_signal {
        //     let action_signal: crate::signals::ActionSignal<T> = crate::signals::ActionSignal {
        //         entry_type: path_string,
        //         action: crate::signals::ActionType::Create,
        //         data: crate::signals::SignalData::Create::<T>(wire_entry.clone()),
        //     };
        //     let signal = convert_to_receiver_signal(action_signal);
        //     let payload = ExternIO::encode(signal)?;
        //     let peers = get_peers()?;
        //     remote_signal(payload, peers)?;
        // }
        Ok(wire_entry)
    }
}