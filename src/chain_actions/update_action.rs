use crate::chain_actions::utils::now_date_time;
use crate::wire_element::WireElement;
use chrono::{DateTime, Datelike, Timelike, Utc};
use hdk::prelude::*;
use holo_hash::{AgentPubKey, EntryHashB64, HeaderHashB64};

#[cfg(feature = "mock")]
use ::mockall::automock;

/// a struct which implements a [update_action](UpdateAction::update_action) method
/// a method is used instead of a function so that it can be mocked to simplify unit testing
#[derive(Debug, PartialEq, Clone)]
pub struct UpdateAction {}
#[cfg_attr(feature = "mock", automock)]
impl UpdateAction {
    /// This will add an update to an entry.
    /// It can also optionally send a signal of this event to all peers supplied in `send_signal_to_peers`
    /// uses `ChainTopOrdering::Relaxed` such that multiple updates can be committed in parallel
    pub fn update_action<T, E, S>(
        &self,
        entry: T,
        header_hash: HeaderHashB64,
        entry_type_id: String,
        send_signal_to_peers: Option<Vec<AgentPubKey>>,
        add_time_path: Option<String>,
    ) -> ExternResult<WireElement<T>>
    where
        Entry: TryFrom<T, Error = E>,
        WasmError: From<E>,
        T: 'static + Clone,
        AppEntryBytes: TryFrom<T, Error = E>,
        S: 'static + From<crate::signals::ActionSignal<T>> + serde::Serialize + std::fmt::Debug,
        E: 'static,
    {
        // calling update instead of update_entry to be able to indicate relaxed chain ordering
        hdk::entry::update(
            header_hash.clone().into(),
            CreateInput::new(
                EntryDefId::App(entry_type_id.clone()),
                Entry::App(entry.clone().try_into()?),
                ChainTopOrdering::Relaxed,
            ),
        )?;
        let entry_address = hash_entry(entry.clone())?;
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
                create_link(time_path.hash()?, entry_address.clone(), ())?;
            }
        }
        let updated_at = sys_time()?;
        // get create time from the header_hash
        let maybe_element = get::<HeaderHash>(header_hash.clone().into(), GetOptions::default())?;
        let created_at = match maybe_element {
            Some(element) => Ok(element.signed_header().header().timestamp()),
            None => Err(WasmError::Guest(String::from("unable to get element from provided header hash"))),
        }?;
        let wire_entry: WireElement<T> = WireElement {
            entry,
            header_hash,
            entry_hash: EntryHashB64::new(entry_address),
            created_at,
            updated_at,
        };
        match send_signal_to_peers {
            None => (),
            Some(vec_peers) => {
                let action_signal: crate::signals::ActionSignal<T> = crate::signals::ActionSignal {
                    entry_type: entry_type_id,
                    action: crate::signals::ActionType::Update,
                    data: crate::signals::SignalData::Update(wire_entry.clone()),
                };
                let signal = S::from(action_signal);
                let payload = ExternIO::encode(signal)?;
                remote_signal(payload, vec_peers)?;
            }
        }
        Ok(wire_entry)
    }
}
