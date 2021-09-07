use didcomm_rs::Jwe;
use rocksdb::{DBWithThreadMode, SingleThreaded, DB};
use serial_test::serial;
use utilities::keypair::get_keypair_set;
use vade::Vade;
use vade_didcomm::{
    datatypes::{
        Base64Container,
        BaseMessage,
        CommKeyPair,
        CommunicationDidDocument,
        DidCommOptions,
        DidDocumentBodyAttachment,
        KeyInformation,
        MessageWithBody,
        VadeDidCommPluginOutput,
    },
    VadeDidComm,
};

const DID_SERVICE_ENDPOINT: &str = "https://evan.network";
const ROCKS_DB_PATH: &str = "./.didcomm_rocks_db";
const DID_EXCHANGE_PROTOCOL_URL: &str = "https://didcomm.org/didexchange/1.0";

pub fn read_db(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let db: DBWithThreadMode<SingleThreaded> = DB::open_default(ROCKS_DB_PATH)?;

    match db.get(key) {
        Ok(Some(result)) => Ok(String::from_utf8(result)?),
        Ok(None) => Err(format!("{0} not found", key).into()),
        Err(e) => Err(format!("Error while loading key: {0}, {1}", key, e).into()),
    }
}

pub fn get_com_keypair(key_agreement_did: &str) -> Result<CommKeyPair, Box<dyn std::error::Error>> {
    let db_result = read_db(&format!("key_agreement_key_{}", key_agreement_did))?;
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    Ok(comm_keypair)
}

fn get_didcomm_receiver_options(
    use_shared_key: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let sign_keypair = get_keypair_set();

    let options: DidCommOptions;
    options = DidCommOptions {
        key_information: Some(KeyInformation::SecretPublic {
            my_secret: sign_keypair.user2_secret.to_bytes(),
            others_public: sign_keypair.user1_pub.to_bytes(),
        }),
        sign_key: Some(sign_keypair.sign_keypair.secret.to_bytes()),
    };

    Ok(serde_json::to_string(&options)?)
}

fn get_didcomm_sender_options(use_shared_key: bool) -> Result<String, Box<dyn std::error::Error>> {
    let sign_keypair = get_keypair_set();

    let options: DidCommOptions;
    options = DidCommOptions {
        key_information: Some(KeyInformation::SecretPublic {
            my_secret: sign_keypair.user1_secret.to_bytes(),
            others_public: sign_keypair.user2_pub.to_bytes(),
        }),
        sign_key: Some(sign_keypair.sign_keypair.secret.to_bytes()),
    };

    Ok(serde_json::to_string(&options)?)
}

async fn get_vade() -> Result<Vade, Box<dyn std::error::Error>> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new()?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
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
    // let db_result = read_db(&format!("comm_keypair_{}_{}", sender, receiver))?;
    // let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    // let pub_key = prepared
    //     .metadata
    //     .get("pub_key")
    //     .ok_or("send DIDComm request does not return pub_key")?
    //     .to_owned();
    // let secret_key = prepared
    //     .metadata
    //     .get("secret_key")
    //     .ok_or("send DIDComm request does not return secret_key")?
    //     .to_owned();
    // let target_pub_key = prepared
    //     .metadata
    //     .get("target_pub_key")
    //     .ok_or("send DIDComm request does not return target_pub_key")?
    //     .to_owned();

    // assert_eq!(target_pub_key, comm_keypair.target_pub_key);
    // assert_eq!(pub_key, comm_keypair.pub_key);
    // assert_eq!(secret_key, comm_keypair.secret_key);

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_request(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(&options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let received: VadeDidCommPluginOutput<
        MessageWithBody<DidDocumentBodyAttachment<Base64Container>>,
    > = serde_json::from_str(result)?;
    let comm_keypair = get_com_keypair(
        &received
            .message
            .id
            .unwrap_or_else(|| "missing id in message".to_string()),
    )?;

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

    // assert_eq!(target_pub_key, comm_keypair.target_pub_key);
    // assert_eq!(pub_key, comm_keypair.pub_key);
    // assert_eq!(secret_key, comm_keypair.secret_key);

    return Ok(());
}

async fn send_response(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
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
    let results = vade.didcomm_send("{}", &exchange_response).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_response(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive("{}", &message).await?;
    let _ = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    // let comm_keypair_sender = get_com_keypair(sender)?;
    // let comm_keypair_receiver = get_com_keypair(receiver)?;

    // assert_eq!(
    //     comm_keypair_sender.target_pub_key,
    //     comm_keypair_receiver.pub_key
    // );

    return Ok(());
}

async fn send_complete(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_complete = format!(
        r#"{{
            "type": "{}/complete",
            "from": "{}",
            "to": ["{}"]
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL, sender, receiver
    );

    let results = vade.didcomm_send("{}", &exchange_complete).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_complete(
    vade: &mut Vade,
    _sender: &str,
    _receiver: &str,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive("{}", &message).await?;
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

    return Ok(());
}

#[tokio::test]
#[serial]
async fn can_do_key_exchange_and_use_shared_secret_for_initial_encryption(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;

    let user_1_did = String::from("did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp");
    let user_2_did = String::from("did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG");
    let sender_options = get_didcomm_sender_options(true)?;
    let receiver_options = get_didcomm_receiver_options(true)?;
    let request_message =
        send_request(&mut vade, &user_1_did, &user_2_did, &sender_options).await?;
    receive_request(
        &mut vade,
        &user_1_did,
        &user_2_did,
        request_message,
        &receiver_options,
    )
    .await?;

    let response_message = send_response(&mut vade, &user_2_did, &user_1_did).await?;
    receive_response(&mut vade, &user_2_did, &user_1_did, response_message).await?;

    let complete_message = send_complete(&mut vade, &user_1_did, &user_2_did).await?;
    receive_complete(&mut vade, &user_2_did, &user_1_did, complete_message).await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_key_exchange_and_use_secret_and_public_for_initial_encryption(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let user_1_did = String::from("did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp");
    let user_2_did = String::from("did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG");
    let sender_options = get_didcomm_sender_options(true)?;
    let receiver_options = get_didcomm_receiver_options(true)?;

    let request_message =
        send_request(&mut vade, &user_1_did, &user_2_did, &sender_options).await?;
    receive_request(
        &mut vade,
        &user_1_did,
        &user_2_did,
        request_message,
        &receiver_options,
    )
    .await?;

    let response_message = send_response(&mut vade, &user_2_did, &user_1_did).await?;
    receive_response(&mut vade, &user_2_did, &user_1_did, response_message).await?;

    let complete_message = send_complete(&mut vade, &user_1_did, &user_2_did).await?;
    receive_complete(&mut vade, &user_1_did, &user_2_did, complete_message).await?;

    Ok(())
}
