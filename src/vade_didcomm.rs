use crate::{AsyncResult, BaseMessage, EncryptedMessage, MessageWithBody, ProtocolHandler, decrypt_message, encrypt_message, get_com_keypair};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use vade::{ResultAsyncifier, VadePlugin, VadePluginResultValue};

big_array! { BigArray; }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommSendOptions {
    pub encryption_key: [u8; 32],
    #[serde(with = "BigArray")]
    pub sign_keypair: [u8; 64],
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommReceiveOptions {
    pub decryption_key: [u8; 32],
    pub sign_public: [u8; 32],
}

#[allow(dead_code)]
pub struct VadeDidComm { }

impl VadeDidComm {
    /// Creates new instance of `VadeDidComm`.
    pub async fn new() -> AsyncResult<VadeDidComm> {
        match env_logger::try_init() {
            Ok(_) | Err(_) => (),
        };
        let vade_didcomm = VadeDidComm { };

        Ok(vade_didcomm)
    }
}

#[async_trait]
impl VadePlugin for VadeDidComm {
    async fn didcomm_send(
        &mut self,
        options: &str,
        message: &str,
    ) -> AsyncResult<VadePluginResultValue<Option<String>>> {
        log::debug!("preparing DIDComm message for being sent");

        let protocol_result = ProtocolHandler::before_send(message).asyncify()?;
        let final_message: String;

        if protocol_result.encrypt {
            // if encryption opts were passed to the message, use the passed encryption keys
            let options = serde_json::from_str::<DidcommSendOptions>(&options);
            if options.is_ok() {
                let parsed_options = options?;
                final_message = encrypt_message(
                    &protocol_result.message,
                    &parsed_options.encryption_key,
                    &parsed_options.sign_keypair,
                ).asyncify()?;
            } else {
                // otherwise use keys from did exchange
                let parsed_message: BaseMessage = serde_json::from_str(message)?;
                let from_did = parsed_message.from.as_ref().ok_or("from is required")?;
                let to_vec = parsed_message.to.as_ref().ok_or("to is required")?;
                let to_did = &to_vec[0];

                // let encoded_keypair = get_com_keypair(
                //     from_did,
                //     to_did,
                // ).asyncify()?;
                // let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
                final_message = protocol_result.message;
            }
        } else {
            final_message = protocol_result.message;
        }

        let send_result = format!(
            r#"{{
                "message": {},
                "metadata": {}
            }}"#,
            final_message,
            protocol_result.metadata,
        );

        return Ok(VadePluginResultValue::Success(Some(send_result)));
    }

    async fn didcomm_receive(
        &mut self,
        options: &str,
        message: &str,
    ) -> AsyncResult<VadePluginResultValue<Option<String>>> {
        log::debug!("handling receival of DIDComm message");

        // check if message is encrypted or not
        let parsed_message = serde_json::from_str::<EncryptedMessage>(message);
        let decrypted: String;
        if parsed_message.is_ok() {
            let options = serde_json::from_str::<DidcommReceiveOptions>(&options)?;
            decrypted = decrypt_message(
                &message,
                &options.decryption_key,
                &options.sign_public,
            ).asyncify()?;
        } else {
            decrypted = String::from(message);
        }

        let protocol_result = ProtocolHandler::after_receive(&decrypted).asyncify()?;
        let receive_result = format!(
            r#"{{
                "message": {},
                "metadata": {}
            }}"#,
            protocol_result.message,
            protocol_result.metadata,
        );

        return Ok(VadePluginResultValue::Success(Some(receive_result)));
    }
}
