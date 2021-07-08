use utilities::keypair::get_keypair_set;
use vade::Vade;
use vade_didcomm::{AsyncResult, EncryptedMessage, Message, ProtocolOutput, VadeDidComm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PingBody {
    response_requested: bool,
}

async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

fn get_send_options(
    encryption_key: &x25519_dalek::SharedSecret,
    sign_keypair: &ed25519_dalek::Keypair,
) -> String {
    let options = format!(
        r#"{{
            "encryptionKey": {:?},
            "signKeypair": {:?}
        }}"#,
        &encryption_key.as_bytes(),
        &sign_keypair.to_bytes(),
    );

    return options;
}

fn get_receive_options(
    decryption_key: &x25519_dalek::SharedSecret,
    sign_keypair: &ed25519_dalek::Keypair,
) -> String {
    let options = format!(
        r#"{{
            "decryptionKey": {:?},
            "signPublic": {:?}
        }}"#,
        &decryption_key.as_bytes(),
        &sign_keypair.public.to_bytes()
    );

    return options;
}

#[tokio::test]
async fn can_be_registered_as_plugin() -> AsyncResult<()> {
    get_vade().await?;

    Ok(())
}

#[tokio::test]
async fn can_prepare_didcomm_message_for_sending() -> AsyncResult<()> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_keypair_set();
    let options = get_send_options(&sign_keypair.user1_shared, &sign_keypair.sign_keypair);
    let payload = format!(
        r#"{{
            "type": "https://didcomm.org/trust_ping/1.0/ping",
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "custom1": "ichi",
            "custom2": "ni",
            "custom3": "san"
        }}"#,
    );
    let results = vade.didcomm_send(&options, &payload).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let parsed: ProtocolOutput<EncryptedMessage> = serde_json::from_str(result)?;

    assert_eq!(parsed.message.other.get("custom1").ok_or("could not field custom1")?, "ichi");

    Ok(())
}

#[tokio::test]
async fn can_decrypt_received_messages() -> AsyncResult<()> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_keypair_set();
    let options = get_send_options(&sign_keypair.user1_shared, &sign_keypair.sign_keypair);

    let payload = format!(
        r#"{{
            "type": "https://didcomm.org/trust_ping/1.0/ping",
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "custom1": "nyuu"
        }}"#,
    );
    let results = vade.didcomm_send(&options, &payload).await?;

    match results.get(0) {
        Some(Some(value)) => {
            let encrypted: ProtocolOutput<EncryptedMessage> = serde_json::from_str(value)?;
            let encrypted_message = serde_json::to_string(&encrypted.message)?;
            let options = get_receive_options(&sign_keypair.user2_shared, &sign_keypair.sign_keypair);
            let results = vade.didcomm_receive(&options, &encrypted_message).await?;
            let result = results
                .get(0)
                .ok_or("no result")?
                .as_ref()
                .ok_or("no value in result")?;
            let parsed: ProtocolOutput<Message> = serde_json::from_str(result)?;
            assert_eq!(
                "https://didcomm.org/trust_ping/1.0/ping",
                parsed.message.r#type.ok_or("could not parse vade decrypted result")?,
            );
            // ensure that send processor was executed
            let body: PingBody = serde_json::from_str(&parsed.message.body)?;
            assert_eq!(body.response_requested, true);
        }
        _ => {
            return Err(Box::from("invalid result from didcomm_send"));
        }
    };

    Ok(())
}

#[tokio::test]
async fn can_receive_unencrypted() -> AsyncResult<()> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_keypair_set();

    let payload = format!(
        r#"{{
            "type": "https://didcomm.org/trust_ping/1.0/ping",
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "custom1": "nyuu"
        }}"#,
    );

    let options = get_receive_options(&sign_keypair.user2_shared, &sign_keypair.sign_keypair);
    let results = vade.didcomm_receive(&options, &payload).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let parsed: Message = serde_json::from_str(result)?;

    assert_eq!(
        "https://didcomm.org/trust_ping/1.0/ping",
        parsed.r#type.ok_or("could not parse vade decrypted result")?,
    );

    Ok(())
}
