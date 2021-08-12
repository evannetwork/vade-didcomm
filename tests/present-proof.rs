use rocksdb::{DBWithThreadMode, SingleThreaded, DB};
use serial_test::serial;
use vade::Vade;
use vade_didcomm::{
    datatypes::{
        BaseMessage, DidCommOptions, EncryptedMessage, KeyInformation, MessageWithBody,
        PresentProofReq, PresentationAttach, PresentationData, VadeDidCommPluginOutput,
        PRESENT_PROOF_PROTOCOL_URL,
    },
    VadeDidComm,
};

const ROCKS_DB_PATH: &str = "./.didcomm_rocks_db";

pub fn read_db(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let db: DBWithThreadMode<SingleThreaded> = DB::open_default(ROCKS_DB_PATH)?;

    match db.get(key) {
        Ok(Some(result)) => Ok(String::from_utf8(result)?),
        Ok(None) => Err(format!("{0} not found", key).into()),
        Err(e) => Err(format!("Error while loading key: {0}, {1}", key, e).into()),
    }
}

pub fn get_presentation(
    from_did: &str,
    to_did: &str,
) -> Result<PresentationData, Box<dyn std::error::Error>> {
    let presentation = read_db(&format!("present_proof_{}_{}", from_did, to_did))?;
    let presentation_data: PresentationData = serde_json::from_str(&presentation)?;
    return Ok(presentation_data);
}

async fn get_vade() -> Result<Vade, Box<dyn std::error::Error>> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new()?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

async fn send_request_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation_data = PresentationData {
        presentation_attach: Some(
            [PresentationAttach {
                id: String::from("some_id"),
                mime_type: String::from("application/json"),
                data: String::from("base 64 data string"),
            }]
            .to_vec(),
        ),
        comment: None,
        presentation_proposal: None,
    };

    let exchange_request = format!(
        r#"{{
            "type": "{}/request-presentation",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"],
            "body": {}
        }}"#,
        PRESENT_PROOF_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&presentation_data)?
    );

    let results = vade.didcomm_send(options, &exchange_request).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let prepared: VadeDidCommPluginOutput<EncryptedMessage> = serde_json::from_str(result)?;

    // let db_result = get_presentation( sender, receiver)?;
    // let request_presentation = prepared.message.body.to_owned().unwrap_or("send DIDComm request does not return presentation request".to_owned());

    // println!("request_presentation {}",request_presentation);

    // assert_eq!(request_presentation, db_result) ;
    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_request_presentation(
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
    let received: VadeDidCommPluginOutput<MessageWithBody<PresentationData>> =
        serde_json::from_str(result)?;
    // let presentation = get_presentation(receiver, sender)?;
    let request_presentation = received
        .message
        .body
        .ok_or("send DIDComm request does not return presentation request".to_owned())?;

    return Ok(());
}

async fn send_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation_data = PresentationData {
        presentation_attach: Some(
            [PresentationAttach {
                id: String::from("some_id"),
                mime_type: String::from("application/json"),
                data: String::from("base 64 data string"),
            }]
            .to_vec(),
        ),
        comment: None,
        presentation_proposal: None,
    };

    let exchange_response = format!(
        r#"{{
            "type": "{}/presentation",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"],
            "body": {}
        }}"#,
        PRESENT_PROOF_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&presentation_data)?
    );
    println!("send response {}", exchange_response);
    let results = vade.didcomm_send(&options, &exchange_response).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<EncryptedMessage> = serde_json::from_str(result)?;

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(&options, &message).await?;
    let _ = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let presentation_sender = get_presentation(sender, receiver)?;
    let presentation_receiver = get_presentation(receiver, sender)?;

    return Ok(());
}

async fn send_ack(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_complete = format!(
        r#"{{
            "type": "{}/ack",
            "from": "{}",
            "to": ["{}"]
        }}"#,
        PRESENT_PROOF_PROTOCOL_URL, sender, receiver
    );
    let results = vade.didcomm_send("{}", &exchange_complete).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<EncryptedMessage> = serde_json::from_str(result)?;

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_ack(
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
        format!("{}/ack", PRESENT_PROOF_PROTOCOL_URL)
    );

    return Ok(());
}

#[tokio::test]
#[serial]
async fn can_do_presentation_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let user_1_did = String::from("did:uknow:d34db33d");
    let user_2_did = String::from("did:uknow:d34db33f");
    let options = String::from("{}");

    let request_message =
        send_request_presentation(&mut vade, &user_1_did, &user_2_did, &options).await?;
    receive_request_presentation(
        &mut vade,
        &user_1_did,
        &user_2_did,
        request_message,
        &options,
    )
    .await?;

    let response_message = send_presentation(&mut vade, &user_2_did, &user_1_did, &options).await?;
    receive_presentation(
        &mut vade,
        &user_2_did,
        &user_1_did,
        response_message,
        &options,
    )
    .await?;

    let complete_message = send_ack(&mut vade, &user_1_did, &user_2_did).await?;
    receive_ack(&mut vade, &user_1_did, &user_2_did, complete_message).await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_proposal_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let user_1_did = String::from("did:uknow:d34db33d");
    let user_2_did = String::from("did:uknow:d34db33f");
    let options = String::from("{}");

    let request_message =
        send_request_presentation(&mut vade, &user_1_did, &user_2_did, &options).await?;
    receive_request_presentation(
        &mut vade,
        &user_1_did,
        &user_2_did,
        request_message,
        &options,
    )
    .await?;

    let response_message = send_presentation(&mut vade, &user_2_did, &user_1_did, &options).await?;
    receive_presentation(
        &mut vade,
        &user_2_did,
        &user_1_did,
        response_message,
        &options,
    )
    .await?;

    let complete_message = send_ack(&mut vade, &user_1_did, &user_2_did).await?;
    receive_ack(&mut vade, &user_1_did, &user_2_did, complete_message).await?;

    Ok(())
}
