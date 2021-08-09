use crate::{
    datatypes::{BaseMessage, DidCommOptions, EncryptedMessage},
    fill_message_id_and_timestamps,
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

pub struct VadeDidComm {}
impl VadeDidComm {
    /// Creates new instance of `VadeDidComm`.
    pub fn new() -> Result<VadeDidComm, Box<dyn std::error::Error>> {
        match env_logger::try_init() {
            Ok(_) | Err(_) => (),
        };
        let vade_didcomm = VadeDidComm {};

        Ok(vade_didcomm)
    }
}

#[async_trait(?Send)]
impl VadePlugin for VadeDidComm {
    /// Prepare a plain DIDComm json message to be sent, including encryption and protocol specific
    /// message enhancement.
    /// The DIDComm options can include a shared secret to encrypt the message with a specific key.
    /// If no key was given and the message should be encrypted (depends on protocol implementation),
    /// the DIDComm keypair from a db will be used.
    ///
    /// # Arguments
    /// * `options` - of type DidcommOptions, used to apply a custom signing_key
    /// * `message` - the plain didcomm message (should be of type datatypes.rs/BaseMessage)
    ///
    /// # Returns
    /// * `VadeDidCommPluginOutput` - stringified datatypes.rs/VadeDidCommPluginOutput contains the
    ///                               final message and protocol step specific metadata
    async fn didcomm_send(
        &mut self,
        options: &str,
        message: &str,
    ) -> Result<VadePluginResultValue<Option<String>>, Box<dyn std::error::Error>> {
        log::debug!("preparing DIDComm message for being sent");

        // run protocol specific logic
        let message_with_id = fill_message_id_and_timestamps(&message)?;
        let protocol_result = ProtocolHandler::before_send(&message_with_id)?;

        // message string, that will be returned
        let final_message: String;

        if protocol_result.encrypt {
            // generate random keypair for message encryption and signing to always have altering
            // signatures
            let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);

            // if shared secret was passed to the options, use this one
            let options = serde_json::from_str::<DidCommOptions>(&options)?;
            let encryption_key: [u8; 32] = match options.key_information {
                Some(crate::datatypes::KeyInformation::SharedSecret { shared_secret }) => {
                    shared_secret
                }
                Some(crate::datatypes::KeyInformation::SecretPublic {
                    my_secret,
                    others_public,
                }) => {
                    let my_secret_deserialized = StaticSecret::from(my_secret);
                    let other_public_deserialized = PublicKey::from(others_public);
                    my_secret_deserialized
                        .diffie_hellman(&other_public_deserialized)
                        .to_bytes()
                }
                None => {
                    // otherwise use keys from DID exchange
                    let parsed_message: BaseMessage = serde_json::from_str(&message_with_id)?;
                    let from_to = get_from_to_from_message(parsed_message)?;
                    let encoded_keypair = get_com_keypair(&from_to.from, &from_to.to)?;
                    let secret_decoded = vec_to_array(hex::decode(encoded_keypair.secret_key)?)?;
                    let target_pub_decoded =
                        vec_to_array(hex::decode(encoded_keypair.target_pub_key)?)?;
                    let secret = StaticSecret::from(secret_decoded);
                    let target_pub_key = PublicKey::from(target_pub_decoded);
                    secret.diffie_hellman(&target_pub_key).to_bytes()
                }
            };
            final_message =
                encrypt_message(&protocol_result.message, &encryption_key, &sign_keypair)?;
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
    /// If no key was given and the message is encrypted the DIDComm keypair from a db will be used.
    ///
    /// # Arguments
    /// * `options` - of type DidcommOptions, used to apply a custom signing_key
    /// * `message` - the plain / encrypted didcomm message (should be of type
    ///               datatypes.rs/BaseMessage / datatypes.rs/EncryptedMessage)
    ///
    /// # Returns
    /// * `VadeDidCommPluginOutput` - stringified datatypes.rs/VadeDidCommPluginOutput contains the
    ///                               final message and protocol step specific metadata
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
            let options = serde_json::from_str::<DidCommOptions>(&options)?;
            let decryption_key: [u8; 32] = match options.key_information {
                Some(crate::datatypes::KeyInformation::SharedSecret { shared_secret }) => {
                    shared_secret
                }
                Some(crate::datatypes::KeyInformation::SecretPublic {
                    my_secret,
                    others_public,
                }) => {
                    let my_secret_deserialized = StaticSecret::from(my_secret);
                    let other_public_deserialized = PublicKey::from(others_public);
                    my_secret_deserialized
                        .diffie_hellman(&other_public_deserialized)
                        .to_bytes()
                }
                None => {
                    // otherwise use keys from DID exchange
                    let base_message = serde_json::from_str::<BaseMessage>(message)?;
                    let from_to = get_from_to_from_message(base_message)?;

                    let encoded_keypair = get_com_keypair(&from_to.to, &from_to.from)?;
                    let secret_decoded = vec_to_array(hex::decode(encoded_keypair.secret_key)?)?;
                    let target_pub_decoded =
                        vec_to_array(hex::decode(encoded_keypair.target_pub_key)?)?;
                    let secret = StaticSecret::from(secret_decoded);
                    let target_pub_key = PublicKey::from(target_pub_decoded);
                    secret.diffie_hellman(&target_pub_key).to_bytes()
                }
            };
            decrypted = decrypt_message(&message, &decryption_key, &hex::decode(signing_pub_key)?)?;
        } else {
            decrypted = String::from(message);
        }

        // run protocol specific logic
        let message_with_id = fill_message_id_and_timestamps(&decrypted)?;
        let protocol_result = ProtocolHandler::after_receive(&message_with_id)?;

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
