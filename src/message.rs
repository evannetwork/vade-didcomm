use serde::{Deserialize, Serialize};
use didcomm_rs::{
    crypto::{CryptoAlgorithm, SignatureAlgorithm},
    Message as DIDCommMessage,
};
use std::{collections::HashMap};

pub struct Decrypted {
    pub body: Message,
    pub message: DIDCommMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    #[serde(default)]
    pub body: String,
    pub from: Option<String>,
    pub kid: Option<String>,
    pub to: Option<Vec<String>>,
    pub r#type: Option<String>,
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
    pub ciphertext: Option<Vec<u8>>,
    pub iv: Option<Vec<u8>>,
    pub id: Option<u64>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
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

impl Message {
    pub fn from_string(message: &str) -> Result<Message, String> {
        let message: Message = serde_json::from_str(message)
            .map_err(|err| {
                format!(
                    "could not get valid message from received data: {}",
                    &err.to_string()
                )
            })?;
        return Ok(message);
    }

    pub fn to_string(self) -> Result<String, String> {
        let message = serde_json::to_string(&self)
            .map_err(|err| {
                format!(
                    "Could not format message to string: {}",
                    &err.to_string()
                )
            })?;
        return Ok(message);
    }

    pub fn decrypt(
        message: &str,
        decryption_key: &[u8],
        sign_public: &[u8],
    ) -> Result<Decrypted, String> {
        let received =
            DIDCommMessage::receive(
                &message,
                Some(decryption_key),
                Some(sign_public),
            ).map_err(|err| {
                format!(
                    "could not decrypt message: {}",
                    &err.to_string()
                )
            })?;

        let body = String::from_utf8(received.body.clone()).map_err(|err| {
            format!(
                "could not get body from message while decrypting message: {}",
                &err.to_string()
            )
        })?;

        return Ok(Decrypted {
            message: received,
            body: Message::from_string(&body)?,
        });
    }

    pub fn encrypt(
        message_string: &str,
        encryption_key: &[u8],
        key_pair: &[u8],
    ) -> Result<Option<String>, String> {
        let mut d_message = DIDCommMessage::new()
            .body(message_string.to_string().as_bytes())
            .as_jwe(&CryptoAlgorithm::XC20P);
        let message = Message::from_string(message_string)?;

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
            d_message = d_message.add_header_field(
                key.to_owned(),
                val.to_string().to_owned(),
            );
        }

        // finally sign and encrypt
        let encrypted = d_message
            .seal_signed(
                encryption_key,
                key_pair,
                SignatureAlgorithm::EdDsa,
            ).map_err(|err| {
                format!(
                    "could not run searl_signed while encrypting message: {}",
                    &err.to_string()
                )
            })?;

        Ok(Some(encrypted))
    }
}
