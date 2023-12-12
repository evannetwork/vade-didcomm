use std::collections::HashMap;
use didcomm_rs::Attachment;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{get_from_to_from_message, utils::hex_option};

pub const DEFAULT_DIDCOMM_SERVICE_ENDPOINT: &str = "http://127.0.0.1:7070/didcomm";

pub trait HasFromAndTo {
    fn get_from_to(&self) -> Result<FromTo, Box<dyn std::error::Error>>;
}

/// Struct for a pub key that will be sent during DID exchange with the users communication DID document.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCommPubKey {
    pub id: String,
    pub public_key_base_58: String,
    pub r#type: Vec<String>,
}

/// Struct for a service definition that will be sent during DID exchange with the users communication DID document.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCommService {
    pub id: String,
    pub r#type: String,
    pub priority: u8,
    pub service_endpoint: String,
    pub recipient_keys: Vec<String>,
}

/// Communication DIDComm object that will be sent to the target user during DID exchange.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunicationDidDocument {
    #[serde(rename(serialize = "@context", deserialize = "@context"))]
    pub context: String,
    pub id: String,
    pub authentication: Vec<String>,
    pub public_key: Vec<DidCommPubKey>,
    pub service: Vec<DidCommService>,
}

/// Basically a set of a to and a from DID
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FromTo {
    pub from: String,
    pub to: String,
}

/// Necessary information for a valid DID exchange request / response extracted from an DIDComm message
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    pub from: String,
    pub to: String,
    pub did_id: String,
    pub pub_key_hex: String,
    pub service_endpoint: String,
}

/// Communication keypair with the complete information to encrypt and decrypt a message from a
/// specific comm partner. Each key is saved as hex encoded u8 array. Please checkout vade_didcomm.rs
/// and did_exchange/request.rs for reference implementations.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommKeyPair {
    pub pub_key: String,
    pub secret_key: String,
    pub key_agreement_key: String,
    pub target_key_agreement_key: String,
    pub target_pub_key: String,
    pub target_service_endpoint: String,
}

/// Specifies all possible message directions.
#[derive(PartialEq)]
pub enum MessageDirection {
    Send,
    Receive,
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
    pub body: HashMap<String, Value>,
    pub from: Option<String>,
    pub r#type: String,
    pub to: Option<Vec<String>>,
}

/// Decrypted message format without dynamic body
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtendedMessage {
    pub body: Option<HashMap<String, Value>>,
    pub created_time: Option<u64>,
    pub expires_time: Option<u64>,
    pub from: Option<String>,
    pub id: Option<String>,
    pub pthid: Option<String>,
    pub r#type: String,
    pub thid: Option<String>,
    pub to: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<Attachment>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
}

/// Object with base64 encoded value
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Base64Container {
    pub base64: String,
}

/// `did_doc~attach` attachment for body field
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidDocumentBodyAttachment<T> {
    #[serde(rename(serialize = "did_doc~attach", deserialize = "did_doc~attach"))]
    pub did_doc_attach: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Decrypted messaged with dynamic body struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageWithBody<T> {
    pub body: Option<T>,
    pub created_time: Option<u64>,
    pub expires_time: Option<u64>,
    pub from: Option<String>,
    pub id: Option<String>,
    pub pthid: Option<String>,
    pub r#type: String,
    pub thid: Option<String>,
    pub to: Option<Vec<String>>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<Attachment>,
}
impl<T> HasFromAndTo for MessageWithBody<T> {
    fn get_from_to(&self) -> Result<FromTo, Box<dyn std::error::Error>> {
        let base_message: BaseMessage = BaseMessage {
            body: HashMap::new(),
            from: self.from.to_owned(),
            r#type: self.r#type.to_owned(),
            to: Some(self.to.clone().ok_or("To DID not provided.")?.to_vec()),
        };

        get_from_to_from_message(&base_message)
    }
}

/// Message format, when a message was encrypted with DIDComm rs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedMessage {
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

/// Generated KeyPair for encryption
#[derive(Serialize, Deserialize)]
pub struct EncryptionKeyPair {
    #[serde(with = "hex")]
    pub secret: [u8; 32],
    #[serde(with = "hex")]
    pub public: [u8; 32],
}

/// Either a computed shared secret or a (local) private key plus a contacts public key
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionKeys {
    #[serde(with = "hex")]
    pub encryption_my_secret: [u8; 32],
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "hex_option")]
    pub encryption_others_public: Option<[u8; 32]>,
}

/// Either a computed shared secret or a (local) private key plus a contacts public key
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigningKeys {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "hex_option")]
    pub signing_my_secret: Option<[u8; 32]>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "hex_option")]
    pub signing_others_public: Option<[u8; 32]>,
}

/// Data contains optional field which have to filled as per
/// attached data requirement (json/base64) as per protocol
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base64: Option<String>,
}

/// Optional parameter that can be passed to vade DIDComm functions to enforce a specific encryption key
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCommOptions {
    pub encryption_keys: Option<EncryptionKeys>,
    pub signing_keys: Option<SigningKeys>,
    pub skip_message_packaging: Option<bool>,
    pub skip_protocol_handling: Option<bool>,
}

/// Output of didcomm_send.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VadeDidCommPluginSendOutput<T, TRaw = serde_json::Value> {
    pub message: T,
    pub message_raw: TRaw,
    pub metadata: HashMap<String, String>,
}

/// Output of didcomm_receive.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VadeDidCommPluginReceiveOutput<T> {
    pub message: T,
    pub metadata: HashMap<String, String>,
}
