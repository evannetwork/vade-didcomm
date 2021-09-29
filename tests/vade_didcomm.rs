use didcomm_rs::Jwe;
use serde::{Deserialize, Serialize};
use utilities::keypair::get_keypair_set;
use vade::Vade;
use vade_didcomm::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody, VadeDidCommPluginOutput},
    VadeDidComm,
};
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PingBody {
    response_requested: bool,
}

async fn get_vade() -> Result<Vade, Box<dyn std::error::Error>> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new()?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

#[tokio::test]
async fn can_be_registered_as_plugin() -> Result<(), Box<dyn std::error::Error>> {
    get_vade().await?;

    Ok(())
}

#[tokio::test]
async fn can_prepare_didcomm_message_for_sending() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_keypair_set();
    let payload = r#"{
        "type": "https://didcomm.org/trust_ping/1.0/ping",
        "to": [ "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG" ],
        "custom1": "ichi",
        "custom2": "ni",
        "custom3": "san"
    }"#;
    let results = vade
        .didcomm_send(&sign_keypair.sender_options_stringified, &payload)
        .await?;
    let _result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    Ok(())
}

#[tokio::test]
async fn can_decrypt_received_messages() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_keypair_set();
    let payload = r#"{
        "type": "https://didcomm.org/trust_ping/1.0/ping",
        "from": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
        "to": [ "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG" ],
        "custom1": "nyuu"
    }"#;
    let results = vade
        .didcomm_send(&sign_keypair.sender_options_stringified, &payload)
        .await?;

    match results.get(0) {
        Some(Some(value)) => {
            let encrypted: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(value)?;
            let encrypted_message = serde_json::to_string(&encrypted.message)?;
            let results = vade
                .didcomm_receive(
                    &sign_keypair.receiver_options_stringified,
                    &encrypted_message,
                )
                .await?;
            let result = results
                .get(0)
                .ok_or("no result")?
                .as_ref()
                .ok_or("no value in result")?;
            let parsed: VadeDidCommPluginOutput<MessageWithBody<PingBody>> =
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
            return Err(Box::from("invalid result from DIDcomm_send"));
        }
    };

    Ok(())
}

#[tokio::test]
async fn can_receive_unencrypted() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_keypair_set();

    let payload = r#"{
        "type": "https://didcomm.org/trust_ping/1.0/ping",
        "from": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
        "to": [ "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG" ],
        "custom1": "nyuu"
    }"#;

    let results = vade
        .didcomm_receive(&sign_keypair.receiver_options_stringified, &payload)
        .await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let parsed: VadeDidCommPluginOutput<BaseMessage> = serde_json::from_str(result)?;

    assert_eq!(
        "https://didcomm.org/trust_ping/1.0/ping",
        parsed.message.r#type,
    );

    Ok(())
}

#[tokio::test]
async fn should_fill_empty_id_and_created_time() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_keypair_set();

    let payload = r#"{
        "type": "https://didcomm.org/trust_ping/1.0/ping",
        "from": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
        "to": [ "did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG" ]
    }"#;
    let results = vade
        .didcomm_receive(&sign_keypair.receiver_options_stringified, &payload)
        .await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let parsed: VadeDidCommPluginOutput<ExtendedMessage> = serde_json::from_str(result)?;

    if parsed.message.id.is_none() {
        return Err(Box::from("Default id was not generated!"));
    }

    if parsed.message.created_time.is_none() {
        return Err(Box::from("Default created_time was not generated!"));
    }

    Ok(())
}
