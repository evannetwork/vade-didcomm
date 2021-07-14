use didcomm_rs::{
    crypto::{CryptoAlgorithm, SignatureAlgorithm},
    Message as DIDCommMessage,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::SyncResult;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtocolOutput<T> {
    pub message: T,
    pub metadata: HashMap<String, String>,
}

macro_rules! apply_optional {
    ($message:ident, $payload:ident, $payload_arg:ident) => {{
        match $payload.$payload_arg {
            Some(value) => {
                $message = $message.$payload_arg(&value);
            }
            _ => (),
        }
    }};
}

pub fn decrypt_message(
    message: &str,
    decryption_key: &[u8],
    sign_public: &[u8],
) -> SyncResult<String> {
    let received = DIDCommMessage::receive(&message, Some(decryption_key), Some(sign_public))
        .map_err(|err| format!("could not decrypt message: {}", &err.to_string()))?;

    let decrypted = String::from_utf8(received.body.clone()).map_err(|err| {
        format!(
            "could not get body from message while decrypting message: {}",
            &err.to_string()
        )
    })?;

    return Ok(decrypted);
}

pub fn encrypt_message(
    message_string: &str,
    encryption_key: &[u8],
    keypair: &ed25519_dalek::Keypair,
) -> SyncResult<String> {
    let mut d_message = DIDCommMessage::new()
        .body(message_string.to_string().as_bytes())
        .as_jwe(&CryptoAlgorithm::XC20P);
    let message: ExtendedMessage = serde_json::from_str(message_string)?;

    // apply optional headers to known sections, use remaining as custom headers
    apply_optional!(d_message, message, from);

    match message.to {
        Some(values) => {
            let to: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
            d_message = d_message.to(&to);
        }
        _ => (),
    };

    // insert custom headers
    for (key, val) in message.other.iter() {
        d_message = d_message.add_header_field(key.to_owned(), val.to_string().to_owned());
    }

    // ensure to set kid to pub key of temporary keypair for encryption / signing
    d_message = d_message.kid(&hex::encode(keypair.public.to_bytes()));

    // finally sign and encrypt
    let encrypted = d_message
        .seal_signed(
            encryption_key,
            &keypair.to_bytes(),
            SignatureAlgorithm::EdDsa,
        )
        .map_err(|err| {
            format!(
                "could not run seal_signed while encrypting message: {}",
                &err.to_string()
            )
        })?;

    return Ok(encrypted);
}
