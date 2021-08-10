use hdk::prelude::*;
use std::fmt;

/// when sending signals, distinguish
/// between "create", "update", and "delete" actions
/// via this enum. Serializes to/from "create" | "update" | "delete"
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
#[serde(from = "UIEnum")]
#[serde(into = "UIEnum")]
pub enum ActionType {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
struct UIEnum(String);

impl From<UIEnum> for ActionType {
    fn from(ui_enum: UIEnum) -> Self {
        match ui_enum.0.as_str() {
            "create" => Self::Create,
            "update" => Self::Update,
            _ => Self::Delete,
        }
    }
}

impl From<ActionType> for UIEnum {
    fn from(action_type: ActionType) -> Self {
        Self(action_type.to_string().to_lowercase())
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Grant unrestricted access for this agent to receive
/// calls to its `recv_remote_signal` endpoint via others
/// calling `remote_signal`
pub fn create_receive_signal_cap_grant() -> ExternResult<()> {
    let mut functions: GrantedFunctions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));

    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;
    Ok(())
}

/// Create 2 special types from an existing type
/// that are optimized as a data structure for sending signals
/// to a UI or client that relate to core Holochain actions for
/// entry types: Create, Update, and Delete
#[macro_export]
macro_rules! signal_types {
    (
      $crud_type:ident
    ) => {
        ::paste::paste! {
          /// Distinguishes between what data structures should be passed
          /// to the UI based on different action types, like create/update/delete
          /// this will be used to send these data structures as signals to the UI
          /// When Create/Update, we will pass the actual new Entry
          /// but when doing Delete we will naturally only pass the HeaderHash
          #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
          // untagged because the useful tagging is done externally on the *Signal object
          // as the tag and action
          #[serde(untagged)]
          pub enum [<$crud_type SignalData>] {
            Create([<$crud_type WireEntry>]),
            Update([<$crud_type WireEntry>]),
            Delete($crate::WrappedHeaderHash),
          }

          /// This will be used to send data events as signals to the UI. All
          /// signals relating to the entry type will share this high level structure, creating consistency.
          /// The `data` field should use the variant (Create/Update/Delete)
          /// that matches the variant for `action`. So if `action` is variant [ActionType::Create](crate::ActionType::Create)
          #[doc = " then `data` should be [" $crud_type "SignalData::Create]."]
          #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
          pub struct [<$crud_type Signal>] {
            pub entry_type: String,
            pub action: $crate::ActionType,
            pub data: [<$crud_type SignalData>],
          }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::create_receive_signal_cap_grant;
    use ::fixt::prelude::*;
    use hdk::prelude::*;

    #[test]
    fn test_create_receive_signal_cap_grant() {
        // set up the mock hdk responses
        let mut mock_hdk = MockHdkT::new();
        // zome info is dynamic so
        // that this is generic and usable in any zome
        let zome_info = fixt!(ZomeInfo);
        mock_hdk
            .expect_zome_info()
            .times(1)
            .return_const(Ok(zome_info.clone()));
        // create_cap_grant calls just `create` under the hood
        let mut functions: GrantedFunctions = BTreeSet::new();
        functions.insert((zome_info.zome_name, "recv_remote_signal".into()));
        let expected = EntryWithDefId::new(
            EntryDefId::CapGrant,
            Entry::CapGrant(CapGrantEntry {
                tag: "".into(),
                // empty access converts to unrestricted
                access: ().into(),
                functions,
            }),
        );
        let header_hash = fixt!(HeaderHash);
        mock_hdk
            .expect_create()
            .with(mockall::predicate::eq(expected))
            .times(1)
            .return_const(Ok(header_hash));
        set_hdk(mock_hdk);
        // call the function we are testing
        let result = create_receive_signal_cap_grant();
        assert_eq!(result.is_ok(), true);
    }
}
