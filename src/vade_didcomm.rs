use crate::{
    message::{Message},
    AsyncResult,
    protocol_handler,
};
use async_trait::async_trait;
use didcomm_rs::{
    crypto::{CryptoAlgorithm, SignatureAlgorithm},
    Message as DIDCommMessage,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap};
use vade::{VadePlugin, VadePluginResultValue};

big_array! { BigArray; }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferOptions {
    pub transfer: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommSendOptions {
    pub encryption_key: [u8; 32],
    #[serde(with = "BigArray")]
    pub sign_keypair: [u8; 64],
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidcommSendPayload {
    pub body: String,
    pub from: Option<String>,
    pub kid: Option<String>,
    pub to: Option<Vec<String>>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) other: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommReceiveOptions {
    pub decryption_key: [u8; 32],
    pub sign_public: [u8; 32],
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommReceiveResult {
    pub message: DIDCommMessage,
    pub body: String,
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

#[allow(dead_code)]
pub struct VadeDidComm {
    signer: String,
    target: String,
}

impl VadeDidComm {
    /// Creates new instance of `VadeDidComm`.
    pub async fn new(
        signer: String,
        target: String,
    ) -> AsyncResult<VadeDidComm> {
        match env_logger::try_init() {
            Ok(_) | Err(_) => (),
        };
        let vade_didcomm = VadeDidComm {
            signer,
            target,
        };

        Ok(vade_didcomm)
    }

    fn decrypt_message(
        &mut self,
        message: &str,
        decryption_key: Option<&[u8]>,
        validation_key: Option<&[u8]>,
    ) -> AsyncResult<DidcommReceiveResult> {
        log::debug!("receiving message");

        let received =
            DIDCommMessage::receive(&message, decryption_key, validation_key).map_err(|err| {
                format!(
                    "could not get valid message from received data: {}",
                    &err.to_string()
                )
            })?;

        let body = String::from_utf8(received.body.clone())?;

        Ok(DidcommReceiveResult {
            message: received,
            body,
        })
    }
}

#[async_trait]
impl VadePlugin for VadeDidComm {
    async fn didcomm_send(
        &mut self,
        options: &str,
        payload: &str,
    ) -> AsyncResult<VadePluginResultValue<Option<String>>> {
        log::debug!("preparing DIDComm message for being sent");

        let options = serde_json::from_str::<DidcommSendOptions>(&options)?;
        let payload = serde_json::from_str::<DidcommSendPayload>(&payload)?;

        // create new message with basic setup
        let mut message = DIDCommMessage::new()
            .body(payload.body.as_bytes())
            .as_jwe(&CryptoAlgorithm::XC20P);

        match payload.to {
            Some(values) => {
                let to: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
                message = message.to(&to);
            }
            _ => (),
        };

        // apply optional headers to known sections, use remaining as custom headers
        apply_optional!(message, payload, from);
        apply_optional!(message, payload, kid);

        // insert custom headers
        for (key, val) in payload.other.iter() {
            message = message.add_header_field(key.to_owned(), val.to_owned());
        }

        // finally sign and encrypt
        let ready_to_send = message
            .seal_signed(
                &options.encryption_key,
                &options.sign_keypair,
                SignatureAlgorithm::EdDsa,
            )
            .unwrap();

        Ok(VadePluginResultValue::Success(Some(ready_to_send)))
    }

    async fn didcomm_receive(
        &mut self,
        options: &str,
        payload: &str,
    ) -> AsyncResult<VadePluginResultValue<Option<String>>> {
        log::debug!("handling receival of DIDComm message");

        let options = serde_json::from_str::<DidcommReceiveOptions>(&options)?;

        let decrypted = self.decrypt_message(
            &payload,
            Some(&options.decryption_key),
            Some(&options.sign_public),
        )?;

        Ok(VadePluginResultValue::Success(Some(serde_json::to_string(
            &decrypted,
        )?)))
    }
}
