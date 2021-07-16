use crate::{
    datatypes::{BaseMessage, DidcommOptions, EncryptedMessage},
    get_from_to_from_message,
    keypair::get_com_keypair,
    message::{decrypt_message, encrypt_message},
    protocol_handler::ProtocolHandler,
    utils::vec_to_array,
};
use async_trait::async_trait;
use k256::elliptic_curve::rand_core::OsRng;
use vade::{VadePlugin, VadePluginResultValue};
use x25519_dalek::{PublicKey, StaticSecret};

big_array! { BigArray; }

pub struct VadeDIDComm {}
impl VadeDIDComm {
    /// Creates new instance of `VadeDIDComm`.
    pub async fn new() -> Result<VadeDIDComm, Box<dyn std::error::Error>> {
        match env_logger::try_init() {
            Ok(_) | Err(_) => (),
        };
        let vade_didcomm = VadeDIDComm {};

        Ok(vade_didcomm)
    }
}

#[async_trait(?Send)]
impl VadePlugin for VadeDIDComm {
    /// Prepare a plain DIDComm json message to be sent, including encryption and protocol specific
    /// message enhancement.
    /// The DIDComm options can include a shared secret to encrypt the message with a specific key.
    /// If no key was given and the message should be encrypted (depends on protocol implementation),
    /// the DIDComm keypair from rocks db will be used.
    async fn didcomm_send(
        &mut self,
        options: &str,
        message: &str,
    ) -> Result<VadePluginResultValue<Option<String>>, Box<dyn std::error::Error>> {
        log::debug!("preparing DIDComm message for being sent");

        // run protocol specific logic
        let protocol_result = ProtocolHandler::before_send(message)?;

        // message string, that will be returned
        let final_message: String;

        if protocol_result.encrypt {
            // generate random keypair for message encryption and signing to always have altering
            // signatures
            let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);

            // if shared secret was passed to the options, use this one
            let options = serde_json::from_str::<DidcommOptions>(&options);
            if options.is_ok() {
                let parsed_options = options?;
                final_message = encrypt_message(
                    &protocol_result.message,
                    &parsed_options.shared_secret,
                    &sign_keypair,
                )?;
            } else {
                // otherwise use keys from DID exchange
                let parsed_message: BaseMessage = serde_json::from_str(message)?;
                let from_to = get_from_to_from_message(parsed_message)?;
                let encoded_keypair = get_com_keypair(&from_to.from, &from_to.to)?;
                let secret_decoded = vec_to_array(hex::decode(encoded_keypair.secret_key)?)?;
                let target_pub_decoded =
                    vec_to_array(hex::decode(encoded_keypair.target_pub_key)?)?;
                let secret = StaticSecret::from(secret_decoded);
                let target_pub_key = PublicKey::from(target_pub_decoded);
                let shared_secret = secret.diffie_hellman(&target_pub_key);

                final_message = encrypt_message(
                    &protocol_result.message,
                    shared_secret.as_bytes(),
                    &sign_keypair,
                )?;
            }
        } else {
            final_message = protocol_result.message;
        }

        let send_result = format!(
            r#"{{
                "message": {},
                "metadata": {}
            }}"#,
            final_message, protocol_result.metadata,
        );

        return Ok(VadePluginResultValue::Success(Some(send_result)));
    }

    /// Receive a plain DIDComm json message, including decryption and protocol specific message parsing.
    /// The DIDComm options can include a shared secret to encrypt the message with a specific key.
    /// If no key was given and the message is encrypted the DIDComm keypair from rocks db will be used.
    async fn didcomm_receive(
        &mut self,
        options: &str,
        message: &str,
    ) -> Result<VadePluginResultValue<Option<String>>, Box<dyn std::error::Error>> {
        log::debug!("handling incoming DIDComm message");

        // run protocol specific logic
        let parsed_message = serde_json::from_str::<EncryptedMessage>(message);

        // message string, that will be returned
        let decrypted: String;

        // if the message is encrypted, try to decrypt it
        if parsed_message.is_ok() {
            let encrypted_message = parsed_message?;
            let signing_pub_key = encrypted_message
                .kid
                .ok_or("kid not set in encrypted message")?;

            // if shared secret was passed to the options, use this one
            let options = serde_json::from_str::<DidcommOptions>(&options);
            if options.is_ok() {
                let parsed_options = options?;
                decrypted = decrypt_message(
                    &message,
                    &parsed_options.shared_secret,
                    &hex::decode(signing_pub_key)?,
                )?;
            } else {
                // otherwise use keys from DID exchange
                let base_message = serde_json::from_str::<BaseMessage>(message)?;
                let from_to = get_from_to_from_message(base_message)?;

                let encoded_keypair = get_com_keypair(&from_to.to, &from_to.from)?;
                let secret_decoded = vec_to_array(hex::decode(encoded_keypair.secret_key)?)?;
                let target_pub_decoded =
                    vec_to_array(hex::decode(encoded_keypair.target_pub_key)?)?;
                let secret = StaticSecret::from(secret_decoded);
                let target_pub_key = PublicKey::from(target_pub_decoded);
                let shared_secret = secret.diffie_hellman(&target_pub_key);

                decrypted = decrypt_message(
                    &message,
                    shared_secret.as_bytes(),
                    &hex::decode(signing_pub_key)?,
                )?;
            }
        } else {
            decrypted = String::from(message);
        }

        // run protocol specific logic
        let protocol_result = ProtocolHandler::after_receive(&decrypted)?;

        let receive_result = format!(
            r#"{{
                "message": {},
                "metadata": {}
            }}"#,
            protocol_result.message, protocol_result.metadata,
        );

        return Ok(VadePluginResultValue::Success(Some(receive_result)));
    }
}
