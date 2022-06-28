mod common;

use common::{get_vade, read_db};
use didcomm_rs::Jwe;
use serial_test::serial;
use utilities::keypair::get_keypair_set;
use uuid::Uuid;
use vade::Vade;
use vade_didcomm::{
    datatypes::{
        Base64Container,
        BaseMessage,
        CommKeyPair,
        DidCommOptions,
        DidDocumentBodyAttachment,
        EncryptionKeyPair,
        EncryptionKeys,
        MessageWithBody,
        VadeDidCommPluginReceiveOutput,
        VadeDidCommPluginSendOutput,
    },
    protocols::did_exchange::{
        datatypes::{ProblemReport, ProblemReportData, UserType},
        DidExchangeOptions,
    },
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
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_request = format!(
        r#"{{
            "type": "{}/request",
            "serviceEndpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"],
            "thid": "{}",
            "body": {{}}
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL, sender, receiver, id,
    );
    let results = vade.didcomm_send(options, &exchange_request).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginSendOutput<Jwe> = serde_json::from_str(result)?;

    let pub_key = prepared
        .metadata
        .get("pubKey")
        .ok_or("send DIDComm request does not return pubKey")?
        .to_owned();
    let secret_key = prepared
        .metadata
        .get("secretKey")
        .ok_or("send DIDComm request does not return secretKey")?
        .to_owned();
    let target_pub_key = prepared
        .metadata
        .get("targetPubKey")
        .ok_or("send DIDComm request does not return targetPubKey")?
        .to_owned();

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let db_result = read_db(&format!("comm_keypair_{}_{}", sender, receiver))?;
            let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

            assert_eq!(target_pub_key, comm_keypair.target_pub_key);
            assert_eq!(pub_key, comm_keypair.pub_key);
            assert_eq!(secret_key, comm_keypair.secret_key);
        } else {}
    }

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
    let received: VadeDidCommPluginReceiveOutput<
        MessageWithBody<DidDocumentBodyAttachment<Base64Container>>,
    > = serde_json::from_str(result)?;

    let target_did = received
        .metadata
        .get("keyAgreementKey")
        .ok_or("no keyAgreementKey")?;

    let pub_key = received
        .metadata
        .get("pubKey")
        .ok_or("send DIDComm request does not return pubKey")?
        .to_owned();
    let secret_key = received
        .metadata
        .get("secretKey")
        .ok_or("send DIDComm request does not return secretKey")?
        .to_owned();
    let target_pub_key = received
        .metadata
        .get("targetPubKey")
        .ok_or("send DIDComm request does not return targetPubKey")?
        .to_owned();

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let comm_keypair = get_com_keypair(target_did)?;

            assert_eq!(target_pub_key, comm_keypair.target_pub_key);
            assert_eq!(pub_key, comm_keypair.pub_key);
            assert_eq!(secret_key, comm_keypair.secret_key);
        } else {}
    }

    Ok(())
}

async fn send_response(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_response = format!(
        r#"{{
            "type": "{}/response",
            "serviceEndpoint": "{}",
            "from": "{}",
            "to": ["{}"],
            "thid": "{}",
            "body": {{}}
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL, DID_SERVICE_ENDPOINT, sender, receiver, id
    );
    let results = vade.didcomm_send(options, &exchange_response).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginSendOutput<Jwe> = serde_json::from_str(result)?;

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

    let received: VadeDidCommPluginReceiveOutput<
        MessageWithBody<DidDocumentBodyAttachment<Base64Container>>,
    > = serde_json::from_str(result)?;
    let receiver_did = received
        .metadata
        .get("keyAgreementKey")
        .ok_or("no keyAgreementKey")?;
    let sender_did = received
        .metadata
        .get("targetKeyAgreementKey")
        .ok_or("no targetKeyAgreementKey")?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let comm_keypair_receiver = get_com_keypair(receiver_did)?;
            let comm_keypair_sender = get_com_keypair(sender_did)?;

            assert_eq!(
                comm_keypair_sender.target_pub_key,
                comm_keypair_receiver.pub_key
            );
        } else {}
    }

    Ok(())
}

async fn send_complete(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_complete = format!(
        r#"{{
            "type": "{}/complete",
            "from": "{}",
            "to": ["{}"],
            "thid": "{}",
            "body": {{}}
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL, sender, receiver, id
    );

    let results = vade.didcomm_send(options, &exchange_complete).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginSendOutput<Jwe> = serde_json::from_str(result)?;

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
    let complete_message: VadeDidCommPluginReceiveOutput<BaseMessage> =
        serde_json::from_str(received)?;

    assert_eq!(
        complete_message.message.r#type,
        format!("{}/complete", DID_EXCHANGE_PROTOCOL_URL)
    );

    Ok(())
}

async fn create_keys(vade: &mut Vade) -> Result<EncryptionKeyPair, Box<dyn std::error::Error>> {
    let results = vade
        .run_custom_function("{}", "create_keys", "{}", "{}")
        .await?;
    let received = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let keys: EncryptionKeyPair = serde_json::from_str(received)?;

    Ok(keys)
}

async fn send_problem_report(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let problem = ProblemReport {
        r#type: format!("{}/problem-report", DID_EXCHANGE_PROTOCOL_URL),
        from: Some(sender.to_string()),
        to: Some([receiver.to_string()].to_vec()),
        id: id.to_string(),
        thid: Some(id.to_string()),
        body: ProblemReportData {
            description: Some(String::from("Request Rejected.")),
            problem_items: None,
            who_retries: None,
            fix_hint: None,
            impact: None,
            r#where: None,
            noticed_time: None,
            tracking_uri: None,
            escalation_uri: None,
            user_type: UserType::Inviter,
        },
    };
    let message_string = serde_json::to_string(&problem).map_err(|e| e.to_string())?;

    let results = vade.didcomm_send(options, &message_string).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginSendOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_problem_report(
    vade: &mut Vade,
    _sender: &str,
    _receiver: &str,
    options: &str,
    message: String,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginReceiveOutput<ProblemReport> = serde_json::from_str(result)?;

    let received_problem = received.message;

    assert_eq!(received_problem.thid.ok_or("Thread id not sent")?, id);

    Ok(())
}

#[tokio::test]
#[serial]
#[cfg(feature = "state_storage")]
async fn can_do_key_exchange_with_auto_generated_keys() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message = send_request(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
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
        &id,
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
        &id,
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

    let id = Uuid::new_v4().to_simple().to_string();
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
        &id,
    )
    .await?;

    let mut options_object: DidExchangeOptions =
        serde_json::from_str(&test_setup.receiver_options_stringified)?;
    options_object.did_exchange_my_secret = Some(receiver_secret_key);
    let options_string = serde_json::to_string(&options_object)?;

    receive_request(&mut vade, request_message, &options_string).await?;

    let mut receiver_options_string = test_setup.receiver_signing_options_stringified.to_owned();
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "state_storage"))] {
            let mut receiver_options_object: DidExchangeOptions =
                serde_json::from_str(&receiver_options_string)?;
            receiver_options_object.did_exchange_my_secret = Some(receiver_secret_key);
            receiver_options_string = serde_json::to_string(&receiver_options_object)?;
        } else {}
    };

    let response_message = send_response(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &receiver_options_string,
        &id,
    )
    .await?;

    let mut sender_options_string = test_setup.sender_signing_options_stringified.to_owned();
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "state_storage"))] {
            let mut sender_options_object: DidExchangeOptions =
                serde_json::from_str(&sender_options_string)?;
            sender_options_object.did_exchange_my_secret = Some(sender_secret_key);
            sender_options_string = serde_json::to_string(&sender_options_object)?;
        } else {}
    };

    receive_response(&mut vade, response_message, &sender_options_string).await?;

    let complete_message = send_complete(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &sender_options_string,
        &id,
    )
    .await?;

    receive_complete(&mut vade, complete_message, &receiver_options_string).await?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
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
        } else {}
    }

    Ok(())
}

#[tokio::test]
#[serial]
#[cfg(feature = "state_storage")]
async fn can_do_key_exchange_with_create_keys() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let alice_keys = create_keys(&mut vade).await?;
    let bob_keys = create_keys(&mut vade).await?;
    let id = Uuid::new_v4().to_simple().to_string();

    let didcomm_options_alice = DidCommOptions {
        encryption_keys: Some(EncryptionKeys {
            encryption_my_secret: alice_keys.secret,
            encryption_others_public: Some(bob_keys.public),
        }),
        signing_keys: None,
        skip_message_packaging: Some(false),
        skip_protocol_handling: Some(false),
    };

    let didcomm_options_bob = DidCommOptions {
        encryption_keys: Some(EncryptionKeys {
            encryption_my_secret: bob_keys.secret,
            encryption_others_public: Some(alice_keys.public),
        }),
        signing_keys: None,
        skip_message_packaging: Some(false),
        skip_protocol_handling: Some(false),
    };

    let sender_options_stringified =
        serde_json::to_string(&didcomm_options_bob).unwrap_or_else(|_| "{}".to_string());

    let receiver_options_stringified =
        serde_json::to_string(&didcomm_options_alice).unwrap_or_else(|_| "{}".to_string());

    let test_setup = get_keypair_set();

    let request_message = send_request(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &sender_options_stringified,
        &id,
    )
    .await?;
    receive_request(&mut vade, request_message, &receiver_options_stringified).await?;

    let response_message = send_response(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &receiver_options_stringified,
        &id,
    )
    .await?;
    receive_response(&mut vade, response_message, &sender_options_stringified).await?;

    let complete_message = send_complete(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &sender_options_stringified,
        &id,
    )
    .await?;
    receive_complete(&mut vade, complete_message, &receiver_options_stringified).await?;

    Ok(())
}

#[tokio::test]
#[serial]
#[cfg(feature = "state_storage")]
async fn can_report_problem() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let alice_keys = create_keys(&mut vade).await?;
    let bob_keys = create_keys(&mut vade).await?;
    let id = Uuid::new_v4().to_simple().to_string();

    let didcomm_options_alice = DidCommOptions {
        encryption_keys: Some(EncryptionKeys {
            encryption_my_secret: alice_keys.secret,
            encryption_others_public: Some(bob_keys.public),
        }),
        signing_keys: None,
        skip_message_packaging: Some(false),
        skip_protocol_handling: Some(false),
    };

    let didcomm_options_bob = DidCommOptions {
        encryption_keys: Some(EncryptionKeys {
            encryption_my_secret: bob_keys.secret,
            encryption_others_public: Some(alice_keys.public),
        }),
        signing_keys: None,
        skip_message_packaging: Some(false),
        skip_protocol_handling: Some(false),
    };

    let sender_options_stringified =
        serde_json::to_string(&didcomm_options_bob).unwrap_or_else(|_| "{}".to_string());

    let receiver_options_stringified =
        serde_json::to_string(&didcomm_options_alice).unwrap_or_else(|_| "{}".to_string());

    let test_setup = get_keypair_set();

    let request_message = send_request(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &sender_options_stringified,
        &id,
    )
    .await?;
    receive_request(&mut vade, request_message, &receiver_options_stringified).await?;

    let response_message = send_response(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &receiver_options_stringified,
        &id,
    )
    .await?;
    receive_response(&mut vade, response_message, &sender_options_stringified).await?;

    let problem_message = send_problem_report(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;

    receive_problem_report(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.receiver_options_stringified,
        problem_message,
        &id,
    )
    .await?;

    Ok(())
}
