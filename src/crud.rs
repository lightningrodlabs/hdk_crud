/// A macro to go quick and easy
/// from having just a Holochain entry definition
/// to having a full create-read-update-delete set of
/// functionality in your Zome, plus "signals" (events).
/// See [example] for a comprehensive look at how this works.
/// ```ignore
/// use hdk::prelude::*;
/// use hdk_crud::*;
///
/// #[hdk_entry(id = "test")]
/// #[derive(Clone, PartialEq)]
/// pub struct Test {
///     pub number: i32,
/// }
/// // ExampleSignal pops out of the crud! macro
/// #[derive(Debug, Serialize, Deserialize, SerializedBytes)]
/// #[serde(untagged)]
/// pub enum TestSignalTypes {
///     Test(TestSignal)
/// }
/// pub fn convert_to_receiver_signal(signal: TestSignal) -> TestSignalTypes {
///     TestSignalTypes::Test(signal)
/// }
/// pub fn get_peers() -> ExternResult<Vec<AgentPubKey>> {
///     Ok(Vec::new())
/// }
/// crud!(
///     Test,
///     test,
///     "test",
///     get_peers,
///     convert_to_receiver_signal,
/// );
/// ```
use hdk::time::sys_time;
use hdk::prelude::ExternResult;
use chrono::{DateTime, Utc, NaiveDateTime};

fn now_date_time() -> ExternResult<::chrono::DateTime<::chrono::Utc>> {
    let time = sys_time()?.as_seconds_and_nanos();

    let date: DateTime<Utc> =
        DateTime::from_utc(NaiveDateTime::from_timestamp(time.0, time.1), Utc);
    Ok(date)
}

#[macro_export]
macro_rules! crud {
    (
      $crud_type:ident, $i:ident, $path:expr, $get_peers:ident, $convert_to_receiver_signal:ident
    ) => {
        ::paste::paste! {
          use ::chrono::{Datelike, Timelike};
          /// This is the &str that can be passed into Path to
          /// find all the entries created using these create functions
          /// which are linked off of this Path.
          pub const [<$i:upper _PATH>]: &str = $path;

          /// Retrieve the Path for these entry types
          /// to which all entries are linked
          pub fn [<get_ $i _path>]() -> Path {
            Path::from([<$i:upper _PATH>])
          }

          


          #[doc ="This is what is expected by a call to [update_" $path "] or [inner_update_" $path "]"]
          #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
          #[serde(rename_all = "camelCase")]
          pub struct [<$crud_type UpdateInput>] {
            pub entry: $crud_type,
            pub header_hash: ::holo_hash::HeaderHashB64,
          }

          /*
            CREATE
          */
          /// This will create an entry and link it off the main Path.
          /// It can also optionally send a signal of this event (by passing `send_signal` value `true`)
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          #[doc="This will be called with `send_signal` as `true` by [create_" $i "]"]
          pub fn [<inner_create_ $i>](entry: $crud_type, send_signal: bool, add_time_path: Option<String>) -> ExternResult<$crate::wire_element::WireElement<[<$crud_type>]>> {
            let address = create_entry(&entry)?;
            let entry_hash = hash_entry(&entry)?;
            let path = [< get_ $i _path >]();
            path.ensure()?;
            let path_hash = path.hash()?;
            create_link(path_hash, entry_hash.clone(), ())?;

            match add_time_path {
              None => (),
              Some(base_component) => {
                // create a time_path
                let date: ::chrono::DateTime<::chrono::Utc> = $crate::crud::now_date_time()?;
                
                let time_path = $crate::datetime_queries::hour_path_from_date(base_component, date.year(), date.month(), date.day(), date.hour());

                time_path.ensure()?;
                create_link(time_path.hash()?,entry_hash.clone(), ())?;
              }
            }
           
            let wire_entry: $crate::wire_element::WireElement<[<$crud_type>]> = $crate::wire_element::WireElement {
              entry,
              header_hash: ::holo_hash::HeaderHashB64::new(address),
              entry_hash: ::hdk::prelude::holo_hash::EntryHashB64::new(entry_hash)
            };
            
            if (send_signal) {
              let action_signal: $crate::signals::ActionSignal<[<$crud_type>]> = $crate::signals::ActionSignal {
                entry_type: $path.to_string(),
                action: $crate::signals::ActionType::Create,
                data: $crate::signals::SignalData::Create::<[<$crud_type>]>(wire_entry.clone()),
              };
              let signal = $convert_to_receiver_signal(action_signal);
              let payload = ExternIO::encode(signal)?;
              let peers = $get_peers()?;
              remote_signal(payload, peers)?;
            }
            Ok(wire_entry)
          }

          #[cfg(not(feature = "exclude_zome_fns"))]
          /// This is the exposed/public Zome function for creating an entry of this type.
          /// This will create an entry and link it off the main Path.
          /// It will send a signal of this event
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          #[doc="This just calls [inner_create_" $i "] with `send_signal` as `true`."]
          #[hdk_extern]
          pub fn [<create_ $i>](entry: $crud_type) -> ExternResult<$crate::wire_element::WireElement<[<$crud_type>]>> {
            [<inner_create_ $i>](entry, true, None)
          }

          /*
            READ
          */
          /// This is the exposed/public Zome function for either fetching ALL or a SPECIFIC list of the entries of the type.
          pub fn [<inner_fetch_ $i s>](fetch_options: $crate::retrieval::FetchOptions, get_options: GetOptions) -> ExternResult<Vec<$crate::wire_element::WireElement<[<$crud_type>]>>> {
            let entries = $crate::retrieval::fetch_entries::<$crud_type>([< get_ $i _path >](), fetch_options, get_options)?;
            Ok(entries)
          }

          #[cfg(not(feature = "exclude_zome_fns"))]
          /// This is the exposed/public Zome function for either fetching ALL or a SPECIFIC list of the entries of the type.
          /// No signals will be sent as a result of calling this.
          /// Notice that it pluralizes the value of `$i`, the second argument to the crud! macro call.
          #[doc="This just calls [inner_fetch_" $i "s]."]
          #[hdk_extern]
          pub fn [<fetch_ $i s>](fetch_options: $crate::retrieval::FetchOptions) -> ExternResult<Vec<$crate::wire_element::WireElement<[<$crud_type>]>>> {
            [<inner_fetch_ $i s>](fetch_options, GetOptions::latest())
          }

          /*
            UPDATE
          */
          /// This will add an update to an entry.
          /// It can also optionally send a signal of this event (by passing `send_signal` value `true`)
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          pub fn [<inner_update_ $i>](update: [<$crud_type UpdateInput>], send_signal: bool) -> ExternResult<$crate::wire_element::WireElement<[<$crud_type>]>> {
            update_entry(update.header_hash.clone().into(), &update.entry)?;
            let entry_address = hash_entry(&update.entry)?;
            let wire_entry: $crate::wire_element::WireElement<[<$crud_type>]> = $crate::wire_element::WireElement {
                entry: update.entry,
                header_hash: update.header_hash,
                entry_hash: ::hdk::prelude::holo_hash::EntryHashB64::new(entry_address)
            };
            if (send_signal) {
              let action_signal: $crate::signals::ActionSignal<[<$crud_type>]> = $crate::signals::ActionSignal {
                entry_type: $path.to_string(),
                action: $crate::signals::ActionType::Update,
                data: $crate::signals::SignalData::Update(wire_entry.clone()),
              };
              let signal = $convert_to_receiver_signal(action_signal);
              let payload = ExternIO::encode(signal)?;
              let peers = $get_peers()?;
              remote_signal(payload, peers)?;
            }
            Ok(wire_entry)
          }

          #[cfg(not(feature = "exclude_zome_fns"))]
          /// This is the exposed/public Zome function for creating an entry of this type.
          /// This will add an update to an entry.
          /// It will send a signal of this event
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          #[doc="This just calls [inner_update_" $i "] with `send_signal` as `true`."]
          #[hdk_extern]
          pub fn [<update_ $i>](update: [<$crud_type UpdateInput>]) -> ExternResult<$crate::wire_element::WireElement<[<$crud_type>]>> {
            [<inner_update_ $i>](update, true)
          }

          /*
            DELETE
          */
          /// This will mark the entry at `address` as "deleted".
          #[doc="It will no longer be returned by [fetch_" $i "s]."]
          /// It can also optionally send a signal of this event (by passing `send_signal` value `true`)
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          #[doc="This will be called with `send_signal` as `true` by [archive_" $i "]"]
          pub fn [<inner_archive_ $i>](address: ::holo_hash::HeaderHashB64, send_signal: bool) -> ExternResult<::holo_hash::HeaderHashB64> {
            delete_entry(address.clone().into())?;
            if (send_signal) {
              let action_signal: $crate::signals::ActionSignal<[<$crud_type>]> = $crate::signals::ActionSignal {
                entry_type: $path.to_string(),
                action: $crate::signals::ActionType::Delete,
                data: $crate::signals::SignalData::Delete::<[<$crud_type>]>(address.clone()),
              };
              let signal = $convert_to_receiver_signal(action_signal);
              let payload = ExternIO::encode(signal)?;
              let peers = $get_peers()?;
              remote_signal(payload, peers)?;
            }
            Ok(address)
          }

          #[cfg(not(feature = "exclude_zome_fns"))]
          /// This is the exposed/public Zome function for archiving an entry of this type.
          /// This will mark the entry at `address` as "deleted".
          #[doc="It will no longer be returned by [fetch_" $i "s]."]
          /// It will send a signal of this event
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          #[doc="This just calls [inner_archive_" $i "] with `send_signal` as `true`."]
          #[hdk_extern]
          pub fn [<archive_ $i>](address: ::holo_hash::HeaderHashB64) -> ExternResult<::holo_hash::HeaderHashB64> {
            [<inner_archive_ $i>](address, true)
          }
        }
    };
}

/// Take a look at this module to get a concrete example
/// of what you need to pass the crud! macro, as well
/// as what you'll get back out of it.
/// Anything that says "NOT GENERATED" is not
/// generated by the crud! macro call, and the rest is
/// It will generate 4 public Zome functions, as well as
/// some inner functions called by those that you can refer to
/// and call elsewhere. The 4 Zome functions in this example would be:
/// [create_example](example::create_example), [fetch_examples](example::create_example), [update_example](example::create_example), and [archive_example](example::create_example).
pub mod example {
    use crate::signals::*;
    use hdk::prelude::*;

    /// NOT GENERATED
    /// This is our example hdk_entry entry
    /// type definition.
    #[hdk_entry(id = "example")]
    #[derive(Clone, PartialEq)]
    pub struct Example {
        pub number: i32,
    }

    /// NOT GENERATED
    /// A high level signal type to unify all the entry type specific
    /// signal types
    #[derive(Debug, Serialize, Deserialize, SerializedBytes)]
    // untagged because the useful tagging is done internally on the ActionSignal objects
    #[serde(untagged)]
    pub enum SignalTypes {
        Example(ActionSignal<Example>),
    }

    /// NOT GENERATED
    /// Signal Receiver
    /// (forwards signals to the UI)
    /// would be handling a
    pub fn recv_remote_signal(signal: ExternIO) -> ExternResult<()> {
        Ok(emit_signal(&signal)?)
    }

    /// NOT GENERATED
    /// This handles the conversion from its predefined type
    /// to some slightly modified type that it should be sent over the
    /// wire as. It is sort of like a pre-signal-fire hook
    /// presenting the chance to do type conversion
    pub fn convert_to_receiver_signal(signal: ActionSignal<Example>) -> SignalTypes {
        SignalTypes::Example(signal)
    }

    /// NOT GENERATED
    /// This handles the fetching of a list of peers to which to send
    /// signals. In this example it's an empty list. Your function
    /// signature should match this function signature.
    pub fn get_peers() -> ExternResult<Vec<AgentPubKey>> {
        Ok(Vec::new())
    }

    crud!(
        Example,
        example,
        "example",
        get_peers,
        convert_to_receiver_signal
    );
}

#[cfg(test)]
mod tests {
    use super::example::*;

    #[test]
    fn it_works() {
        let e: Example = Example { number: 2 };
        assert_eq!(e, Example { number: 2 });
    }
}
