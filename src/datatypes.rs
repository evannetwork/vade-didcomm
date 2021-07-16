use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub const DID_EXCHANGE_PROTOCOL_URL: &str = "https://didcomm.org/didexchange/1.0";

/// Struct for a pub key that will be sent during did exchange with the users communication did document.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DIDCommPubKey {
    pub id: String,
    pub public_key_base_58: String,
    pub r#type: Vec<String>,
}

/// Struct for a service definition that will be sent during did exchange with the users communication did document.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DIDCommService {
    pub id: String,
    pub r#type: String,
    pub priority: u8,
    pub service_endpoint: String,
    pub recipient_keys: Vec<String>,
}

/// Communication didcomm object that will be sent to the target user during did exchange.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunicationDidDocument {
    #[serde(rename(serialize = "@context", deserialize = "@context"))]
    pub context: String,
    pub id: String,
    pub authentication: Vec<String>,
    pub public_key: Vec<DIDCommPubKey>,
    pub service: Vec<DIDCommService>,
}

/// Basically a set of a to and a from did
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FromTo {
    pub from: String,
    pub to: String,
}

/// Necessary information for a valid did exchange request / response extracted from an didcomm message
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    pub from: String,
    pub to: String,
    pub pub_key_hex: String,
    pub service_endpoint: String,
}

/// Communication keypair with the complete information to encrypt and decrypt a message from a
/// specific comm partner. Each key is saved as hex encoded u8 array. Please checkout vade_didcomm.rs
/// and did_exchange/request.rs for reference implementations.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommKeyPair {
    pub pub_key: String,
    pub secret_key: String,
    pub target_pub_key: String,
    pub target_service_endpoint: String,
}

/// Specifies all possible message directions.
#[derive(PartialEq)]
pub enum MessageDirection {
    SEND,
    RECEIVE,
}

/// Output of a protocol step. Specifies, if a message should be encrypted. Metadata is generic stringified
/// json, that contains protocol step specific information.
pub struct ProtocolHandleOutput {
    pub direction: MessageDirection,
    pub encrypt: bool,
    pub protocol: String,
    pub metadata: String,
    pub message: String,
    pub step: String,
}

/// Base message with only the type (used for protocol handling to analyze only the message type)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageWithType {
    pub r#type: String,
}

/// Decrypted message format without dynamic body
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseMessage {
    pub from: Option<String>,
    pub r#type: String,
    pub to: Option<Vec<String>>,
}

/// Decrypted message format without dynamic body
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtendedMessage {
    pub body: Option<HashMap<String, Value>>,
    pub from: Option<String>,
    pub id: Option<String>,
    pub pthid: Option<String>,
    pub r#type: String,
    pub thid: Option<String>,
    pub to: Option<Vec<String>>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
}

/// Decrypted messaged with dynamic body struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageWithBody<T> {
    pub body: Option<T>,
    pub from: Option<String>,
    pub id: Option<String>,
    pub pthid: Option<String>,
    pub r#type: String,
    pub thid: Option<String>,
    pub to: Option<Vec<String>>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
}

/// Message format, when a message was encrypted with didcomm rs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedMessage {
    #[serde(default)]
    pub from: Option<String>,
    pub kid: Option<String>,
    pub to: Option<Vec<String>>,
    pub r#type: Option<String>,
    pub ciphertext: Vec<u8>,
    pub iv: Vec<u8>,
    pub id: Option<u64>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
}

/// Optional parameter that can be passed to vade didcomm functions to enforce a specific encryption key
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommOptions {
    pub shared_secret: [u8; 32],
}

/// Output of didcomm_send or didcomm_receive.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VadeDIDCommPluginOutput<T> {
    pub message: T,
    pub metadata: HashMap<String, String>,
}
