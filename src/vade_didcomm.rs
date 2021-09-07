use crate::{datatypes::{BaseMessage, DidCommOptions, ExtendedMessage}, fill_message_id_and_timestamps, get_from_to_from_message, keypair::{get_com_keypair, get_key_agreement_key}, message::{decrypt_message, encrypt_message}, protocol_handler::ProtocolHandler, utils::vec_to_array};
use async_trait::async_trait;
use didcomm_rs::Jwe;
use k256::elliptic_curve::rand_core::OsRng;
use vade::{VadePlugin, VadePluginResultValue};
use x25519_dalek::{StaticSecret};

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
        let mut protocol_result = ProtocolHandler::before_send(&options, &message_with_id)?;

        // message string, that will be returned
        let final_message: String;

        if protocol_result.encrypt {
            // generate random keypair for message encryption and signing to always have altering
            // signatures

            let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
            // if shared secret was passed to the options, use this one
            let options = serde_json::from_str::<DidCommOptions>(&options)?;
            // let sign_secret = ed25519_dalek::SecretKey::from_bytes(&options.sign_key);

            let encryption_key: [u8; 32] = match options.key_information {
                Some(crate::datatypes::KeyInformation::SecretPublic {
                    my_secret,
                    others_public: _others_public,
                }) => {
                    my_secret
                }
                None => {
                    // otherwise use keys from DID exchange
                    let parsed_message: BaseMessage = serde_json::from_str(&message_with_id)?;
                    let from_to = get_from_to_from_message(parsed_message)?;
                    let mut encoded_keypair =  get_key_agreement_key(&from_to.from);
                    if encoded_keypair.is_err() {
                        // when we dont find a  key agreement key, try to get the stored keypair
                        encoded_keypair = get_com_keypair(&from_to.from, &from_to.to);
                        if encoded_keypair.is_err() {
                            return Err(Box::from("No keypair found"));
                        }
                        encoded_keypair = get_key_agreement_key(&encoded_keypair?.key_agreement_key);
                    }
                    let keypair = encoded_keypair?;
                    let secret_decoded = vec_to_array(hex::decode(keypair.secret_key)?)?;

                    // when we have a key agreement key, adjust the "to" field to the key agreement
                    let mut parsed_message: ExtendedMessage = serde_json::from_str(&protocol_result.message)?;

                    log::debug!("adjusting from: {}  to: {}", keypair.key_agreement_key, keypair.target_key_agreement_key);
                    parsed_message.to = Some(vec![keypair.target_key_agreement_key]);
                    parsed_message.from = Some(keypair.key_agreement_key);
                    protocol_result.message = serde_json::to_string(&parsed_message)?;

                    StaticSecret::from(secret_decoded).to_bytes()
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
        let parsed_message = serde_json::from_str::<Jwe>(message);

        // message string, that will be returned
        let decrypted: String;

        // if the message is encrypted, try to decrypt it
        if parsed_message.is_ok() {
            // if shared secret was passed to the options, use this one
            let options = serde_json::from_str::<DidCommOptions>(&options)?;
            let decryption_key: [u8; 32] = match options.key_information {
                Some(crate::datatypes::KeyInformation::SecretPublic {
                    my_secret,
                    others_public: _others_public,
                }) => {
                    my_secret
                }
                None => {
                    // otherwise use keys from DID exchange
                    let parsed_message = parsed_message?;
                    let from = parsed_message.protected.unwrap_or_default().skid.unwrap_or_default();
                    let recipient = &parsed_message.recepients.unwrap_or_default()[0];
                    let to = recipient.header.kid.as_ref().unwrap();
                    log::debug!("fetching kak for from: {}  to: {}", to, from);
                    let mut encoded_keypair = get_key_agreement_key(&to);
                    if encoded_keypair.is_err() {
                        // when we dont find a stored keypair, try to get the key agreement key
                        log::debug!("fetching kak for {}", to);
                        encoded_keypair = get_com_keypair(to, &from);
                        if encoded_keypair.is_err() {
                            return Err(Box::from("No keypair found"));
                        }
                    }
                    let secret_decoded = vec_to_array(hex::decode(encoded_keypair?.secret_key)?)?;
                    StaticSecret::from(secret_decoded).to_bytes()
                }
            };
            decrypted = decrypt_message(&message, &decryption_key, &decryption_key)?;
        } else {
            decrypted = String::from(message);
        }

        // run protocol specific logic
        let message_with_id = fill_message_id_and_timestamps(&decrypted)?;
        let protocol_result = ProtocolHandler::after_receive(&options, &message_with_id)?;

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
