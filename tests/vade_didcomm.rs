use serde::{Deserialize, Serialize};
use utilities::keypair::get_keypair_set;
use vade::Vade;
use vade_didcomm::{
    datatypes::{BaseMessage, EncryptedMessage, MessageWithBody, VadeDIDCommPluginOutput},
    AsyncResult, VadeDIDComm,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PingBody {
    response_requested: bool,
}

async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDIDComm::new().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

fn get_didcomm_options(shared_secret: &x25519_dalek::SharedSecret) -> String {
    let options = format!(
        r#"{{
            "sharedSecret": {:?}
        }}"#,
        &shared_secret.as_bytes(),
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
    let options = get_didcomm_options(&sign_keypair.user1_shared);
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

    let parsed: VadeDIDCommPluginOutput<EncryptedMessage> = serde_json::from_str(result)?;
    let custom_field = parsed
        .message
        .other
        .get("custom1")
        .ok_or("could not get custom field custom1")?;

    assert_eq!(custom_field, "ichi");

    Ok(())
}

#[tokio::test]
async fn can_decrypt_received_messages() -> AsyncResult<()> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_keypair_set();
    let options = get_didcomm_options(&sign_keypair.user1_shared);

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
            let encrypted: VadeDIDCommPluginOutput<EncryptedMessage> = serde_json::from_str(value)?;
            let encrypted_message = serde_json::to_string(&encrypted.message)?;
            let options = get_didcomm_options(&sign_keypair.user2_shared);
            let results = vade.didcomm_receive(&options, &encrypted_message).await?;
            let result = results
                .get(0)
                .ok_or("no result")?
                .as_ref()
                .ok_or("no value in result")?;
            let parsed: VadeDIDCommPluginOutput<MessageWithBody<PingBody>> =
                serde_json::from_str(result)?;
            assert_eq!(
                "https://didcomm.org/trust_ping/1.0/ping",
                parsed.message.r#type,
            );
            // ensure that send processor was executed
            assert_eq!(
                parsed
                    .message
                    .body
                    .ok_or("no body filled")?
                    .response_requested,
                true
            );
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

    let options = get_didcomm_options(&sign_keypair.user2_shared);
    let results = vade.didcomm_receive(&options, &payload).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let parsed: VadeDIDCommPluginOutput<BaseMessage> = serde_json::from_str(result)?;

    assert_eq!(
        "https://didcomm.org/trust_ping/1.0/ping",
        parsed.message.r#type,
    );

    Ok(())
}
