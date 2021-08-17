use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
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
}

/// Message format, when a message was encrypted with DIDComm rs.
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

/// Either a computed shared secret or a (local) private key plus a contacts public key
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum KeyInformation {
    #[serde(rename_all = "camelCase")]
    SharedSecret {
        #[serde(with = "hex")]
        shared_secret: [u8; 32],
    },
    #[serde(rename_all = "camelCase")]
    SecretPublic {
        #[serde(with = "hex")]
        my_secret: [u8; 32],
        #[serde(with = "hex")]
        others_public: [u8; 32],
    },
}

/// Optional parameter that can be passed to vade DIDComm functions to enforce a specific encryption key
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCommOptions {
    pub key_information: Option<KeyInformation>,
}

/// Output of didcomm_send or didcomm_receive.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VadeDidCommPluginOutput<T> {
    pub message: T,
    pub metadata: HashMap<String, String>,
}

/// This structure is required to be present in all the steps of Present-Proof protocol for send and receive directions.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresentProofReq {
    pub r#type: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub presentation_data: Option<PresentationData>,
}

/// PresentationAttach contains all the fields required for request-presentation and presentation steps.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PresentationAttach {
    pub r#type: String,
    pub id: String,
    pub mime_type: String,
    pub data: String,
}

/// Presentation preview structure is sent by prover to propose alternate presentation. 
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PresentationPreview {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub attribute: Option<Vec<Attribute>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub predicate: Option<Vec<Predicate>>,
}

/// Attributes structure for PresentationPreview request. 
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Attribute{
    pub name: String,
    pub cred_def_id: String,
    pub mime_type: String,
    pub value: String,
    pub referent: String,
}

/// Predicate structure for PresentationPreview request.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Predicate{
    pub name: String,
    pub cred_def_id: String,
    pub predicate: String,
    pub threshold: u64,
}

/// PresentationData structure contains optional fields to be exchanged for all the steps of Present-Proof steps.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PresentationData{
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub presentation_attach: Option<Vec<PresentationAttach>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub presentation_proposal: Option<PresentationPreview>,
}


/// Problem report structure contains fields which are required for reporting problem
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProblemReport{
    pub r#type: String,
    pub from: Option<String>,
    pub to: Option<Vec<String>>,
    pub id: String,
    pub thid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_items: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_retries: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#where: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noticed_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excalation_uri: Option<String>,
}

/// Ack structure contains fields which are sent to 
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Ack{
    pub from: Option<String>,
    pub to: Option<Vec<String>>,
    pub r#type: String,
    pub id: String,
    pub status: String,
    pub thid: Option<String>,
}