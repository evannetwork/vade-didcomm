use crate::datatypes::ExtendedMessage;
use didcomm_rs::{
    crypto::{CryptoAlgorithm, SignatureAlgorithm},
    Message as DIDCommMessage
};

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

/// Encrypt a stringified plain message, with a given encryption_key and a ed25519_dalek keypair using
/// DIDComm rs. (checkout vade_didcomm.rs or tests/message.rs for example usage)
/// Note: Ensure to always create new signing_key pairs to have altering results. Encryption key
/// should be the shared_secret.
///
/// # Arguments
/// * `message` - message string (should match message.rs/EncryptedMessage)
/// * `encryption_key` - encryption public key (usually the shared_secret)
/// * `keypair` - signing key_pair (ed25519_dalek keypair)
///
/// # Returns
/// * `String` - encrypted stringified message
pub fn encrypt_message(
    message_string: &str,
    encryption_key: &[u8],
    keypair: &ed25519_dalek::Keypair,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut d_message = DIDCommMessage::new()
        .set_body(&message_string.to_string())
        .as_jwe(&CryptoAlgorithm::XC20P);
    let message: ExtendedMessage = serde_json::from_str(message_string)?;

    // apply optional headers to known sections, use remaining as custom headers
    apply_optional!(d_message, message, from);

    if let Some(values) = message.to {
        let to: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
        d_message = d_message.to(&to);
    }

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

    Ok(encrypted)
}

/// Decrypt a stringified encrypted message, with a given decryption_key and signing key using
/// DIDComm rs. (checkout vade_didcomm.rs or tests/message.rs for example usage)
///
/// # Arguments
/// * `message` - message string (should match message.rs/EncryptedMessage)
/// * `decryption_key` - decryption public key (usually the shared_secret)
/// * `sign_public` - signing public key (usually delivered within the encrypted message kid field)
///
/// # Returns
/// * `String` - decrypted stringified message
pub fn decrypt_message(
    message: &str,
    decryption_key: &[u8],
    _sign_public: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    let received = DIDCommMessage::receive(&message, decryption_key)
        .map_err(|err| format!("could not decrypt message: {}", &err.to_string()))?;

    let decrypted = received.get_body().map_err(|err| {
        format!(
            "could not get body from message while decrypting message: {}",
            &err.to_string()
        )
    })?;

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    extern crate utilities;

    use crate::datatypes::{MessageWithBody};

    use super::*;
    use didcomm_rs::Jwe;
    use serde::{Deserialize, Serialize};
    use utilities::keypair::get_keypair_set;
    use base58::FromBase58;
    use ed25519_dalek::{
        SecretKey,
        PublicKey,
        Keypair
    };
    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct TestBody {
        test: bool,
    }

    #[test]
    fn can_encrypt_message() -> Result<(), Box<dyn std::error::Error>> {
        let sign_keypair = get_keypair_set();
        let payload = r#"{
                "body": {"test": true},
                "custom1": "ichi",
                "custom2": "ni",
                "custom3": "san",
                "to": [ "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG" ],
                "from": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
                "type": "test"
            }"#
        .to_string();

        let encrypted = encrypt_message(
            &payload,
            &sign_keypair.user1_secret.to_bytes(),
            &sign_keypair.sign_keypair,
        )?;
        let _: Jwe = serde_json::from_str(&encrypted)?;

        Ok(())
    }

    #[test]
    fn can_decrypt_message() -> Result<(), Box<dyn std::error::Error>> {
        let payload = r#"{
                "body": {"test": true},
                "custom1": "ichi",
                "custom2": "ni",
                "custom3": "san",
                "to": [ "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG" ],
                "from": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
                "type": "test"
            }"#
        .to_string();
        let alice_private = "6QN8DfuN9hjgHgPvLXqgzqYE3jRRGRrmJQZkd5tL8paR".from_base58().unwrap();
        let bobs_private = "HBTcN2MrXNRj9xF9oi8QqYyuEPv3JLLjQKuEgW9oxVKP".from_base58().unwrap();
        let secret_key: SecretKey = SecretKey::from_bytes(alice_private.as_slice())?;
        let alice_public: PublicKey = (&secret_key).into();

        let keypair = Keypair::from_bytes([secret_key.as_ref(), alice_public.as_ref()].concat().as_slice())?;
        let encrypted = encrypt_message(
            &payload,
            &alice_private,
            &keypair,
        )?;

        let decrypted = decrypt_message(
            &encrypted,
            &bobs_private,
            &alice_public.to_bytes(),
        )?;

        let decryped_parsed: MessageWithBody<TestBody> = serde_json::from_str(&decrypted)?;

        assert_eq!(decryped_parsed.body.ok_or("body not available")?.test, true);

        Ok(())
    }
}
