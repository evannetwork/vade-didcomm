use async_trait::async_trait;
use didcomm_rs::Jwe;
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use k256::elliptic_curve::rand_core::OsRng;
use vade::{VadePlugin, VadePluginResultValue};
use x25519_dalek::StaticSecret;

use crate::{
    datatypes::{
        BaseMessage,
        DidCommOptions,
        EncryptionKeyPair,
        EncryptionKeys,
        ExtendedMessage,
        MessageDirection,
        ProtocolHandleOutput,
    },
    fill_message_id_and_timestamps,
    get_from_to_from_message,
    keypair::{get_com_keypair, get_key_agreement_key},
    message::{decrypt_message, encrypt_message},
    protocol_handler::ProtocolHandler,
    utils::{read_raw_message_from_db, vec_to_array, write_raw_message_to_db},
};

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
    /// Runs a custom function, currently supports
    ///
    /// - `create_new_keys` to create a new key pair to be used for DIDCOMM communication.
    ///
    /// # Arguments
    ///
    /// * `_method` - not required, can be left empty
    /// * `function` - currently supports `create_new_keys`
    /// * `_options` - not required, can be left empty
    /// * `_payload` - not required, can be left empty
    ///
    /// # Returns
    /// * `Option<String>>` - created key pair
    async fn run_custom_function(
        &mut self,
        _method: &str,
        function: &str,
        _options: &str,
        payload: &str,
    ) -> Result<VadePluginResultValue<Option<String>>, Box<dyn std::error::Error>> {
        match function {
            "create_keys" => {
                let secret_key = StaticSecret::new(OsRng);
                let pub_key = x25519_dalek::PublicKey::from(&secret_key);

                let enc_key_pair = EncryptionKeyPair {
                    secret: secret_key.to_bytes(),
                    public: pub_key.to_bytes(),
                };
                Ok(VadePluginResultValue::Success(Some(serde_json::to_string(
                    &enc_key_pair,
                )?)))
            }
            "query_didcomm_messages" => {
                let mut message_values = payload.split('_');
                let prefix = message_values.next().ok_or("Invalid message prefix")?;
                let thid = message_values.next().ok_or("Invalid message thid")?;
                let message_id = message_values.next().ok_or("Invalid message id")?;

                let db_result = read_raw_message_from_db(prefix, thid, message_id)?;
                let result = serde_json::to_string(&db_result)?;

                Ok(VadePluginResultValue::Success(Some(result)))
            }
            _ => Ok(VadePluginResultValue::Ignored),
        }
    }

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

        let options_parsed = serde_json::from_str::<DidCommOptions>(options)?;
        let message_with_id = fill_message_id_and_timestamps(message)?;

        let mut protocol_result = match options_parsed.skip_protocol_handling {
            None | Some(false) => {
                // run protocol specific logic
                ProtocolHandler::before_send(options, &message_with_id)?
            }
            _ => ProtocolHandleOutput {
                direction: MessageDirection::Send,
                encrypt: true,
                protocol: "".to_string(),
                metadata: "{}".to_string(),
                message: message.to_owned(),
                step: "".to_string(),
            },
        };

        // keep a copy of unencrypted message
        let message_raw = &message_with_id;
        // store unencrypted raw message in db
        write_raw_message_to_db(message_raw)?;

        // message string, that will be returned
        let final_message: String;

        if protocol_result.encrypt && !matches!(options_parsed.skip_message_packaging, Some(true)) {
            let encryption_keys: EncryptionKeys;
            if options_parsed.encryption_keys.is_some() {
                encryption_keys = options_parsed
                    .encryption_keys
                    .ok_or("encryption_keys is missing in options parameter")?;
            } else {
                // otherwise use keys from DID exchange
                let parsed_message: BaseMessage = serde_json::from_str(message)?;
                let from_to = get_from_to_from_message(&parsed_message)?;
                let mut encoded_keypair = get_key_agreement_key(&from_to.from);
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
                let public_decoded = vec_to_array(hex::decode(keypair.target_pub_key)?)?;

                // when we have a key agreement key, adjust the "to" field to the key agreement
                let mut parsed_message: ExtendedMessage =
                    serde_json::from_str(&protocol_result.message)?;

                log::debug!(
                    "adjusting from: {} to:{}",
                    keypair.key_agreement_key,
                    keypair.target_key_agreement_key
                );
                parsed_message.to = Some(vec![keypair.target_key_agreement_key]);
                parsed_message.from = Some(keypair.key_agreement_key);
                protocol_result.message = serde_json::to_string(&parsed_message)?;

                encryption_keys = EncryptionKeys {
                    encryption_my_secret: StaticSecret::from(secret_decoded).to_bytes(),
                    encryption_others_public: Some(public_decoded),
                };
            }

            let signing_keypair;
            if let Some(signing_keys_input) = options_parsed.signing_keys {
                let secret_key = SecretKey::from_bytes(
                    &signing_keys_input
                        .signing_my_secret
                        .ok_or("No signing secret key provided")?,
                )?;
                signing_keypair = Some(Keypair {
                    public: PublicKey::from(&secret_key),
                    secret: secret_key,
                });
            } else {
                signing_keypair = None;
            }
            final_message = encrypt_message(
                &protocol_result.message,
                &encryption_keys.encryption_my_secret,
                encryption_keys
                    .encryption_others_public
                    .as_ref()
                    .map(|v| &v[..]),
                signing_keypair,
            )?;
        } else {
            final_message = protocol_result.message;
        }

        let send_result = format!(
            r#"{{
                "message": {},
                "messageRaw": {},
                "metadata": {}
            }}"#,
            final_message, message_raw, protocol_result.metadata,
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

        let options_parsed = serde_json::from_str::<DidCommOptions>(options)?;
        let parsed_message = serde_json::from_str::<Jwe>(message);

        // message string, that will be returned
        let decrypted: String;

        // if the message is encrypted, try to decrypt it
        if parsed_message.is_ok() {
            // if shared secret was passed to the options, use this one
            let decryption_keys: EncryptionKeys;
            if options_parsed.encryption_keys.is_some() {
                decryption_keys = options_parsed
                    .encryption_keys
                    .ok_or("encryption_keys is missing")?;
            } else {
                // otherwise use keys from DID exchange
                let parsed_message = parsed_message?;
                let from = parsed_message
                    .protected
                    .unwrap_or_default()
                    .skid
                    .unwrap_or_default();
                let recipient = &parsed_message.recipients.unwrap_or_default()[0];
                let to = recipient.header.kid.as_ref().unwrap();
                log::debug!("fetching kak for from: {} to: {}", to, from);
                let mut encoded_keypair = get_key_agreement_key(to);
                if encoded_keypair.is_err() {
                    // when we don't find a stored keypair, try to get the key agreement key
                    log::debug!("fetching kak for {}", to);
                    encoded_keypair = get_com_keypair(to, &from);
                    if encoded_keypair.is_err() {
                        return Err(Box::from("No keypair found"));
                    }
                }
                let keypair = encoded_keypair?;
                let mut target_pub_key = None;
                if !keypair.target_pub_key.is_empty() {
                    target_pub_key = Some(vec_to_array(hex::decode(keypair.target_pub_key)?)?);
                }
                decryption_keys = EncryptionKeys {
                    encryption_my_secret: vec_to_array(hex::decode(keypair.secret_key)?)?,
                    encryption_others_public: target_pub_key,
                };
            }
            let signing_others_public = options_parsed
                .signing_keys
                .map(|keys| keys.signing_others_public)
                .flatten();
            decrypted = decrypt_message(
                message,
                Some(&decryption_keys.encryption_my_secret),
                decryption_keys
                    .encryption_others_public
                    .as_ref()
                    .map(|v| &v[..]),
                signing_others_public.as_ref().map(|v| &v[..]),
            )?;
        } else {
            decrypted = String::from(message);
        }

        // run protocol specific logic
        let message_with_id = fill_message_id_and_timestamps(&decrypted)?;
        // store unencrypted raw message in db
        write_raw_message_to_db(&message_with_id)?;

        let protocol_result = match options_parsed.skip_protocol_handling {
            None | Some(false) => {
                // run protocol specific logic
                ProtocolHandler::after_receive(options, &message_with_id)?
            }
            _ => ProtocolHandleOutput {
                direction: MessageDirection::Receive,
                encrypt: true,
                protocol: "".to_string(),
                metadata: "{}".to_string(),
                message: message_with_id.to_string(),
                step: "".to_string(),
            },
        };

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
