

#[macro_export]
macro_rules! crud {
    (
      $crud_type:ident, $i:ident, $path:expr, $get_peers:ident, $convert_to_receiver_signal:ident
    ) => {

        $crate::paste::paste! {
          pub const [<$i:upper _PATH>]: &str = $path;

          #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
          pub struct [<$crud_type WireEntry>] {
            pub entry: $crud_type,
            pub address: $crate::WrappedHeaderHash,
            pub entry_address: $crate::WrappedEntryHash
          }

          // this will be used to send these data structures as signals to the UI
          #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
          // untagged because the useful tagging is done externally on the *Signal object
          // as the tag and action
          #[serde(untagged)]
          pub enum [<$crud_type SignalData>] {
            Create([<$crud_type WireEntry>]),
            Update([<$crud_type WireEntry>]),
            Delete($crate::WrappedHeaderHash),
          }

          // this will be used to send these data structures as signals to the UI
          #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
          pub struct [<$crud_type Signal>] {
            pub entry_type: String,
            pub action: $crate::ActionType,
            pub data: [<$crud_type SignalData>],
          }

          #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
          pub struct [<$crud_type UpdateInput>] {
            pub entry: $crud_type,
            pub address: $crate::WrappedHeaderHash,
          }

          impl From<$crate::EntryAndHash<$crud_type>> for [<$crud_type WireEntry>] {
            fn from(entry_and_hash: $crate::EntryAndHash<$crud_type>) -> Self {
              [<$crud_type WireEntry>] {
                entry: entry_and_hash.0,
                address: $crate::WrappedHeaderHash(entry_and_hash.1),
                entry_address: $crate::WrappedEntryHash(entry_and_hash.2)
              }
            }
          }

          #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
          pub struct [<Vec $crud_type WireEntry>](pub Vec<[<$crud_type WireEntry>]>);

          /*
            CREATE
          */
          pub fn [<inner_create_ $i>](entry: $crud_type, send_signal: bool) -> ExternResult<[<$crud_type WireEntry>]> {
            let address = create_entry(&entry)?;
            let entry_hash = hash_entry(&entry)?;
            let path = Path::from([<$i:upper _PATH>]);
            path.ensure()?;
            let path_hash = path.hash()?;
            create_link(path_hash, entry_hash.clone(), ())?;
            let wire_entry = [<$crud_type WireEntry>] {
              entry,
              address: $crate::WrappedHeaderHash(address),
              entry_address: $crate::WrappedEntryHash(entry_hash)
            };
            if (send_signal) {
              let signal = $convert_to_receiver_signal([<$crud_type Signal>] {
                entry_type: $path.to_string(),
                action: $crate::ActionType::Create,
                data: [<$crud_type SignalData>]::Create(wire_entry.clone()),
              });
              let payload = ExternIO::encode(signal)?;
              let peers = $get_peers()?;
              remote_signal(payload, peers)?;
            }
            Ok(wire_entry)
          }

          #[hdk_extern]
          pub fn [<create_ $i>](entry: $crud_type) -> ExternResult<[<$crud_type WireEntry>]> {
            [<inner_create_ $i>](entry, true)
          }

          /*
            READ
          */
          pub fn [<inner_fetch_ $i s>](get_options: GetOptions) -> ExternResult<[<Vec $crud_type WireEntry>]> {
            let path_hash = Path::from([<$i:upper _PATH>]).hash()?;
            let entries = $crate::fetch_links::<$crud_type, [<$crud_type WireEntry>]>(path_hash, get_options)?;
            Ok([<Vec $crud_type WireEntry>](entries))
          }

          #[hdk_extern]
          pub fn [<fetch_ $i s>](_: ()) -> ExternResult<[<Vec $crud_type WireEntry>]> {
            [<inner_fetch_ $i s>](GetOptions::latest())
          }

          /*
            UPDATE
          */
          pub fn [<inner_update_ $i>](update: [<$crud_type UpdateInput>], send_signal: bool) -> ExternResult<[<$crud_type WireEntry>]> {
            update_entry(update.address.0.clone(), &update.entry)?;
            let entry_address = hash_entry(&update.entry)?;
            let wire_entry = [<$crud_type WireEntry>] {
                entry: update.entry,
                address: update.address,
                entry_address: $crate::WrappedEntryHash(entry_address)
            };
            if (send_signal) {
              let signal = $convert_to_receiver_signal([<$crud_type Signal>] {
                entry_type: $path.to_string(),
                action: $crate::ActionType::Update,
                data: [<$crud_type SignalData>]::Update(wire_entry.clone()),
              });
              let payload = ExternIO::encode(signal)?;
              let peers = $get_peers()?;
              remote_signal(payload, peers)?;
            }
            Ok(wire_entry)
          }

          #[hdk_extern]
          pub fn [<update_ $i>](update: [<$crud_type UpdateInput>]) -> ExternResult<[<$crud_type WireEntry>]> {
            [<inner_update_ $i>](update, true)
          }

          /*
            DELETE
          */
          pub fn [<inner_archive_ $i>](address: $crate::WrappedHeaderHash, send_signal: bool) -> ExternResult<$crate::WrappedHeaderHash> {
            delete_entry(address.0.clone())?;
            if (send_signal) {
              let signal = $convert_to_receiver_signal([<$crud_type Signal>] {
                entry_type: $path.to_string(),
                action: $crate::ActionType::Delete,
                data: [<$crud_type SignalData>]::Delete(address.clone()),
              });
              let payload = ExternIO::encode(signal)?;
              let peers = $get_peers()?;
              remote_signal(payload, peers)?;
            }
            Ok(address)
          }

          #[hdk_extern]
          pub fn [<archive_ $i>](address: $crate::WrappedHeaderHash) -> ExternResult<$crate::WrappedHeaderHash> {
            [<inner_archive_ $i>](address, true)
          }
        }
    };
}