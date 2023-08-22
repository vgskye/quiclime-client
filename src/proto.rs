use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum ServerboundControlMessage {
    RequestDomainAssignment,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum ClientboundControlMessage {
    UnknownMessage,
    DomainAssignmentComplete { domain: String },
    RequestMessageBroadcast { message: String },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BrokerResponse {
    pub id: String,
    pub host: String,
    pub port: u16,
}
