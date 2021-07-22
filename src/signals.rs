
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