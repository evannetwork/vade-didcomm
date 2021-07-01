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
    pub body: String,
    pub from: Option<String>,
    pub kid: Option<String>,
    pub to: Option<Vec<String>>,
    pub r#type: Option<String>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) other: HashMap<String, String>,
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
                    "could not get valid message from received data: {}",
                    &err.to_string()
                )
            })?;

        let body = String::from_utf8(received.body.clone()).map_err(|err| {
            format!(
                "could not get valid message from received data: {}",
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

        // insert custom headers
        for (key, val) in message.other.iter() {
            d_message = d_message.add_header_field(
                key.to_owned(),
                val.to_string().to_owned(),
            );
        }

        // finally sign and encrypt
        let ready_to_send = d_message
            .seal_signed(
                encryption_key,
                key_pair,
                SignatureAlgorithm::EdDsa,
            )
            .unwrap();

        Ok(Some(ready_to_send))
    }

    pub fn to_string(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
