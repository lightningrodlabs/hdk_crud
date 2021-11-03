
use chrono::{DateTime, NaiveDateTime, Utc};
use hdk::prelude::ExternResult;
use hdk::time::sys_time;


pub fn now_date_time() -> ExternResult<::chrono::DateTime<::chrono::Utc>> {
    let time = sys_time()?.as_seconds_and_nanos();

    let date: DateTime<Utc> =
        DateTime::from_utc(NaiveDateTime::from_timestamp(time.0, time.1), Utc);
    Ok(date)
}
/// A macro to go quick and easy
/// from having just a Holochain entry definition
/// to having a full create-read-update-delete set of
/// functionality in your Zome, "signals" (events), as well as
/// time based queries.
/// See [example] for a comprehensive look at how this works.
/// ```ignore
/// use hdk::prelude::*;
/// use hdk_crud::*;
/// #[hdk_entry(id = "test")]
/// #[derive(Clone, PartialEq)]
/// pub struct Test {
///     pub number: i32,
/// }
/// // TestSignal pops out of the crud! macro
/// #[derive(Debug, Serialize, Deserialize, SerializedBytes)]
/// #[serde(untagged)]
/// pub enum SignalTypes {
/// Example(ActionSignal<Example>),
/// }
/// impl From<ActionSignal<Example>> for SignalTypes {
///     fn from(value: ActionSignal<Example>) -> Self {
///         SignalTypes::Example(value)
///     }
/// }
/// 
/// pub fn recv_remote_signal(signal: ExternIO) -> ExternResult<()> {
///     Ok(emit_signal(&signal)?)
/// }
/// 
/// pub fn get_peers() -> ExternResult<Vec<AgentPubKey>> {
///     Ok(Vec::new())
/// }
/// 
/// crud!(
///     Example,
///     example,
///     "example",
///     get_peers,
///     SignalTypes
/// );
/// ```
#[macro_export]
macro_rules! crud {
    (
      $crud_type:ident, $i:ident, $path:expr, $get_peers:ident, $signal_type:ident
    ) => {
        ::paste::paste! {

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

          #[cfg(not(feature = "exclude_zome_fns"))]
          /// This is the exposed/public Zome function for creating an entry of this type.
          /// This will create an entry and link it off the main Path.
          /// It will send a signal of this event
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          #[hdk_extern]
          pub fn [<create_ $i>](entry: $crud_type) -> ExternResult<$crate::wire_element::WireElement<[<$crud_type>]>> {
            let create_action = $crate::chain_actions::create_action::CreateAction {};
            create_action.create_action::<$crud_type, ::hdk::prelude::WasmError, $signal_type> (
              entry,
              [< get_ $i _path >](),
              $path.to_string(),
              true,
              None,
              $get_peers()?,
            )
          }

          /*
            READ
          */

          #[cfg(not(feature = "exclude_zome_fns"))]
          /// This is the exposed/public Zome function for either fetching ALL or a SPECIFIC list of the entries of the type.
          /// No signals will be sent as a result of calling this.
          /// Notice that it pluralizes the value of `$i`, the second argument to the crud! macro call.
          #[hdk_extern]
          pub fn [<fetch_ $i s>](fetch_options: $crate::retrieval::inputs::FetchOptions) -> ExternResult<Vec<$crate::wire_element::WireElement<[<$crud_type>]>>> {
            let fetch_action = $crate::chain_actions::fetch_action::FetchAction {};
            let fetch_entries = $crate::retrieval::fetch_entries::FetchEntries {};
            let fetch_links = $crate::retrieval::fetch_links::FetchLinks {};
            fetch_action.fetch_action::<$crud_type, ::hdk::prelude::WasmError>(
                &fetch_entries,
                &fetch_links,
              fetch_options,
              GetOptions::latest(),
              [< get_ $i _path >](),
            )
          }

          /*
            UPDATE
          */

          #[cfg(not(feature = "exclude_zome_fns"))]
          /// This is the exposed/public Zome function for creating an entry of this type.
          /// This will add an update to an entry.
          /// It will send a signal of this event
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          #[hdk_extern]
          pub fn [<update_ $i>](update: [<$crud_type UpdateInput>]) -> ExternResult<$crate::wire_element::WireElement<[<$crud_type>]>> {
            let update_action = $crate::chain_actions::update_action::UpdateAction {};
            update_action.update_action::<$crud_type, ::hdk::prelude::WasmError, $signal_type>(
              update.entry,
              update.header_hash,
              $path.to_string(),
              true,
              $get_peers()?,
            )
          }

          /*
            DELETE
          */

          #[cfg(not(feature = "exclude_zome_fns"))]
          /// This is the exposed/public Zome function for archiving an entry of this type.
          /// This will mark the entry at `address` as "deleted".
          #[doc="It will no longer be returned by [fetch_" $i "s]."]
          /// It will send a signal of this event
          /// to all peers returned by the `get_peers` call given during the macro call to `crud!`
          #[hdk_extern]
          pub fn [<delete_ $i>](address: ::holo_hash::HeaderHashB64) -> ExternResult<::holo_hash::HeaderHashB64> {
            let delete_action = $crate::chain_actions::delete_action::DeleteAction {};
            delete_action.delete_action::<$crud_type, ::hdk::prelude::WasmError, $signal_type>(
              address,
              $path.to_string(),
              true,
              $get_peers()?,
            )
          }
        }
    };
}

/// Take a look at this module to get a concrete example
/// of what you need to pass to the crud! macro, as well
/// as what you'll get back out of it.
/// Anything that says "NOT GENERATED" is not
/// generated by the crud! macro call, and the rest is.
/// It will generate 4 public Zome functions
/// The 4 Zome functions in this example would be:
/// [create_example](example::create_example), [fetch_examples](example::create_example), [update_example](example::create_example), and [delete_example](example::create_example).
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
    /// signal types. Must implement the `From<ActionSignal<Example>>` trait
    #[derive(Debug, Serialize, Deserialize, SerializedBytes)]
    // untagged because the useful tagging is done internally on the ActionSignal objects
    #[serde(untagged)]
    pub enum SignalTypes {
        Example(ActionSignal<Example>),
    }
    impl From<ActionSignal<Example>> for SignalTypes {
        fn from(value: ActionSignal<Example>) -> Self {
            SignalTypes::Example(value)
        }
    }

    /// NOT GENERATED
    /// Signal Receiver
    /// (forwards signals to the UI)
    /// would be handling a
    pub fn recv_remote_signal(signal: ExternIO) -> ExternResult<()> {
        Ok(emit_signal(&signal)?)
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
        SignalTypes
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
