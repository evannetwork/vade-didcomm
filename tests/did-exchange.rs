mod common;

use common::{get_vade, read_db};
use didcomm_rs::Jwe;
use serial_test::serial;
use utilities::keypair::get_keypair_set;
use vade::Vade;
use vade_didcomm::{
    datatypes::{
        Base64Container,
        BaseMessage,
        CommKeyPair,
        DidDocumentBodyAttachment,
        MessageWithBody,
        VadeDidCommPluginOutput,
    },
    protocols::did_exchange::DidExchangeOptions,
};

const DID_SERVICE_ENDPOINT: &str = "https://evan.network";
const DID_EXCHANGE_PROTOCOL_URL: &str = "https://didcomm.org/didexchange/1.0";

pub fn get_com_keypair(key_agreement_did: &str) -> Result<CommKeyPair, Box<dyn std::error::Error>> {
    let db_result = read_db(&format!("key_agreement_key_{}", key_agreement_did))?;
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    Ok(comm_keypair)
}

async fn send_request(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_request = format!(
        r#"{{
            "type": "{}/request",
            "serviceEndpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"]
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL, sender, receiver
    );
    let results = vade.didcomm_send(options, &exchange_request).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;
    let db_result = read_db(&format!("comm_keypair_{}_{}", sender, receiver))?;
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    let pub_key = prepared
        .metadata
        .get("pub_key")
        .ok_or("send DIDComm request does not return pub_key")?
        .to_owned();
    let secret_key = prepared
        .metadata
        .get("secret_key")
        .ok_or("send DIDComm request does not return secret_key")?
        .to_owned();
    let target_pub_key = prepared
        .metadata
        .get("target_pub_key")
        .ok_or("send DIDComm request does not return target_pub_key")?
        .to_owned();

    assert_eq!(target_pub_key, comm_keypair.target_pub_key);
    assert_eq!(pub_key, comm_keypair.pub_key);
    assert_eq!(secret_key, comm_keypair.secret_key);

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_request(
    vade: &mut Vade,
    message: String,
    options: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let received: VadeDidCommPluginOutput<
        MessageWithBody<DidDocumentBodyAttachment<Base64Container>>,
    > = serde_json::from_str(result)?;
    let target_did = received
        .metadata
        .get("key_agreement_key")
        .ok_or("no key_agreement_key")?;
    let comm_keypair = get_com_keypair(target_did)?;

    let pub_key = received
        .metadata
        .get("pub_key")
        .ok_or("send DIDComm request does not return pub_key")?
        .to_owned();
    let secret_key = received
        .metadata
        .get("secret_key")
        .ok_or("send DIDComm request does not return secret_key")?
        .to_owned();
    let target_pub_key = received
        .metadata
        .get("target_pub_key")
        .ok_or("send DIDComm request does not return target_pub_key")?
        .to_owned();

    assert_eq!(target_pub_key, comm_keypair.target_pub_key);
    assert_eq!(pub_key, comm_keypair.pub_key);
    assert_eq!(secret_key, comm_keypair.secret_key);

    Ok(())
}

async fn send_response(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_response = format!(
        r#"{{
            "type": "{}/response",
            "serviceEndpoint": "{}",
            "from": "{}",
            "to": ["{}"]
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL, DID_SERVICE_ENDPOINT, sender, receiver
    );
    let results = vade.didcomm_send(options, &exchange_response).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_response(
    vade: &mut Vade,
    message: String,
    options: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<
        MessageWithBody<DidDocumentBodyAttachment<Base64Container>>,
    > = serde_json::from_str(result)?;
    let receiver_did = received
        .metadata
        .get("key_agreement_key")
        .ok_or("no key_agreement_key")?;
    let sender_did = received
        .metadata
        .get("target_key_agreement_key")
        .ok_or("no target_key_agreement_key")?;
    let comm_keypair_receiver = get_com_keypair(receiver_did)?;
    let comm_keypair_sender = get_com_keypair(sender_did)?;

    assert_eq!(
        comm_keypair_sender.target_pub_key,
        comm_keypair_receiver.pub_key
    );

    Ok(())
}

async fn send_complete(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_complete = format!(
        r#"{{
            "type": "{}/complete",
            "from": "{}",
            "to": ["{}"]
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL, sender, receiver
    );

    let results = vade.didcomm_send(options, &exchange_complete).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_complete(
    vade: &mut Vade,
    message: String,
    options: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let received = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let complete_message: VadeDidCommPluginOutput<BaseMessage> = serde_json::from_str(received)?;

    assert_eq!(
        complete_message.message.r#type,
        format!("{}/complete", DID_EXCHANGE_PROTOCOL_URL)
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_key_exchange_with_auto_generated_keys() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let request_message = send_request(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
    )
    .await?;
    receive_request(
        &mut vade,
        request_message,
        &test_setup.receiver_options_stringified,
    )
    .await?;

    let response_message = send_response(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.receiver_signing_options_stringified,
    )
    .await?;
    receive_response(
        &mut vade,
        response_message,
        &test_setup.sender_signing_options_stringified,
    )
    .await?;

    let complete_message = send_complete(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_signing_options_stringified,
    )
    .await?;
    receive_complete(
        &mut vade,
        complete_message,
        &test_setup.receiver_signing_options_stringified,
    )
    .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_key_exchange_pregenerated_keys() -> Result<(), Box<dyn std::error::Error>> {
    let sender_secret_key: [u8; 32] = x25519_dalek::StaticSecret::from([1; 32]).to_bytes();
    let receiver_secret_key: [u8; 32] = x25519_dalek::StaticSecret::from([2; 32]).to_bytes();

    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let mut options_object: DidExchangeOptions =
        serde_json::from_str(&test_setup.sender_options_stringified)?;
    options_object.did_exchange_my_secret = Some(sender_secret_key);
    let options_string = serde_json::to_string(&options_object)?;
    let request_message = send_request(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &options_string,
    )
    .await?;

    let mut options_object: DidExchangeOptions =
        serde_json::from_str(&test_setup.receiver_options_stringified)?;
    options_object.did_exchange_my_secret = Some(receiver_secret_key);
    let options_string = serde_json::to_string(&options_object)?;
    receive_request(&mut vade, request_message, &options_string).await?;

    let response_message = send_response(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.receiver_signing_options_stringified,
    )
    .await?;
    receive_response(
        &mut vade,
        response_message,
        &test_setup.sender_signing_options_stringified,
    )
    .await?;

    let complete_message = send_complete(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_signing_options_stringified,
    )
    .await?;
    receive_complete(
        &mut vade,
        complete_message,
        &test_setup.receiver_signing_options_stringified,
    )
    .await?;

    // check sender key
    let db_result = read_db(&format!(
        "comm_keypair_{}_{}",
        &test_setup.user1_did, &test_setup.user2_did
    ))?;
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;
    assert_eq!(comm_keypair.secret_key, hex::encode(sender_secret_key));

    // check receiver key
    let db_result = read_db(&format!(
        "comm_keypair_{}_{}",
        &test_setup.user2_did, &test_setup.user1_did
    ))?;
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;
    assert_eq!(comm_keypair.secret_key, hex::encode(receiver_secret_key));

    Ok(())
}
