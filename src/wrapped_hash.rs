
use hdk::prelude::*;

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
struct UIStringHash(String);

/// A version of an [AgentPubKey] that will automatically
/// serialize and deserialize to/from string based hashes for use on your
/// client side. It is used by functions throughout this library. You may wish to use it
/// as a type for your fields on an entry type, if trying to reference an [AgentPubKey].
#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
#[serde(try_from = "UIStringHash")]
#[serde(into = "UIStringHash")]
pub struct WrappedAgentPubKey(pub AgentPubKey);

impl WrappedAgentPubKey {
  pub fn new(agent_pubkey: AgentPubKey) -> Self {
      Self(agent_pubkey)
  }
}

/// A version of an [HeaderHash] that will automatically
/// serialize and deserialize to/from string based hashes for use on your
/// client side. It is used by functions throughout this library. You may wish to use it
/// as a type for your fields on an entry type, if trying to reference an [HeaderHash].
#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
#[serde(try_from = "UIStringHash")]
#[serde(into = "UIStringHash")]
pub struct WrappedHeaderHash(pub HeaderHash);

impl WrappedHeaderHash {
  pub fn new(header_hash: HeaderHash) -> Self {
      Self(header_hash)
  }
}

/// A version of an [EntryHash] that will automatically
/// serialize and deserialize to/from string based hashes for use on your
/// client side. It is used by functions throughout this library. You may wish to use it
/// as a type for your fields on an entry type, if trying to reference an [EntryHash].
#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
#[serde(try_from = "UIStringHash")]
#[serde(into = "UIStringHash")]
pub struct WrappedEntryHash(pub EntryHash);

impl TryFrom<UIStringHash> for WrappedAgentPubKey {
    type Error = String;
    fn try_from(ui_string_hash: UIStringHash) -> Result<Self, Self::Error> {
        match AgentPubKey::try_from(ui_string_hash.0) {
            Ok(address) => Ok(Self(address)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}
impl From<WrappedAgentPubKey> for UIStringHash {
    fn from(wrapped_agent_pub_key: WrappedAgentPubKey) -> Self {
        Self(wrapped_agent_pub_key.0.to_string())
    }
}

impl TryFrom<UIStringHash> for WrappedHeaderHash {
    type Error = String;
    fn try_from(ui_string_hash: UIStringHash) -> Result<Self, Self::Error> {
        match HeaderHash::try_from(ui_string_hash.0) {
            Ok(address) => Ok(Self(address)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}
impl From<WrappedHeaderHash> for UIStringHash {
    fn from(wrapped_header_hash: WrappedHeaderHash) -> Self {
        Self(wrapped_header_hash.0.to_string())
    }
}

impl TryFrom<UIStringHash> for WrappedEntryHash {
    type Error = String;
    fn try_from(ui_string_hash: UIStringHash) -> Result<Self, Self::Error> {
        match EntryHash::try_from(ui_string_hash.0) {
            Ok(address) => Ok(Self(address)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}
impl From<WrappedEntryHash> for UIStringHash {
    fn from(wrapped_entry_hash: WrappedEntryHash) -> Self {
        Self(wrapped_entry_hash.0.to_string())
    }
}