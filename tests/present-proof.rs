use rocksdb::{DBWithThreadMode, SingleThreaded, DB};
use serial_test::serial;
use uuid::Uuid;
use vade::Vade;
use vade_didcomm::{
    datatypes::{EncryptedMessage, MessageWithBody, VadeDidCommPluginOutput},
    protocols::present_proof::datatypes::{
        Ack, Attribute, Predicate, PresentationAttach, PresentationData, PresentationPreview,
        ProblemReport, State, UserType,
    },
    VadeDidComm,
};

const ROCKS_DB_PATH: &str = "./.didcomm_rocks_db";
const PRESENT_PROOF_PROTOCOL_URL: &str = "https://didcomm.org/present-proof/1.0";

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
    thid: &str,
    state: State,
) -> Result<PresentationData, Box<dyn std::error::Error>> {
    let presentation = read_db(&format!(
        "present_proof_{}_{}_{}_{}",
        from_did, to_did, state, thid
    ))?;
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
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation_data = PresentationData {
        state: State::PresentationRequested,
        presentation_attach: Some(
            [PresentationAttach {
                r#type: String::from("https://didcomm.org/present-proof/1.0/request-presentation"),
                id: id.to_string(),
                mime_type: String::from("application/json"),
                data: String::from("YmFzZSA2NCBkYXRhIHN0cmluZw"),
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
            "body": {},
            "thid": "{}"
        }}"#,
        PRESENT_PROOF_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&presentation_data)?,
        id
    );

    let results = vade.didcomm_send(options, &exchange_request).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let prepared: VadeDidCommPluginOutput<EncryptedMessage> = serde_json::from_str(result)?;

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_request_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(&options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let received: VadeDidCommPluginOutput<MessageWithBody<PresentationData>> =
        serde_json::from_str(result)?;

    let request_presentation = received
        .message
        .body
        .ok_or("send DIDComm request does not return presentation request".to_owned())?;

    let attached_req = request_presentation
        .presentation_attach
        .ok_or("Presentation request not attached")?;
    let presentation_data = attached_req.get(0).ok_or("Request body is invalid")?;
    let req_data_saved = get_presentation(sender, receiver, id, request_presentation.state)?;
    let attached_req_saved = req_data_saved
        .presentation_attach
        .ok_or("Presentation request not attached")?;
    let presentation_data_saved = attached_req_saved.get(0).ok_or("Request body is invalid")?;

    assert_eq!(presentation_data.data, presentation_data_saved.data);

    return Ok(());
}

async fn send_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation_data = PresentationData {
        state: State::PresentationSent,
        presentation_attach: Some(
            [PresentationAttach {
                r#type: String::from("https://didcomm.org/present-proof/1.0/presentation"),
                id: id.to_string(),
                mime_type: String::from("application/json"),
                data: String::from("YmFzZSA2NCBkYXRhIHN0cmluZw"),
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
            "body": {},
            "thid": "{}"
        }}"#,
        PRESENT_PROOF_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&presentation_data)?,
        id
    );
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
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(&options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<MessageWithBody<PresentationData>> =
        serde_json::from_str(result)?;

    let received_presentation = received
        .message
        .body
        .ok_or("send DIDComm request does not return presentation request".to_owned())?;

    let state = received_presentation.state;
    let attached_presentation = received_presentation
        .presentation_attach
        .ok_or("Presentation request not attached")?;
    let presentation_data = attached_presentation
        .get(0)
        .ok_or("Request body is invalid")?;

    let req_data_saved = get_presentation(sender, receiver, id, state)?;
    let attached_presentation_saved = req_data_saved
        .presentation_attach
        .ok_or("Presentation request not attached")?;
    let presentation_data_saved = attached_presentation_saved
        .get(0)
        .ok_or("Request body is invalid")?;

    assert_eq!(presentation_data.data, presentation_data_saved.data);

    return Ok(());
}

async fn send_presentation_proposal(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation_data = PresentationData {
        state: State::PresentationProposed,
        presentation_proposal: Some(PresentationPreview {
            attribute: Some(
                [Attribute {
                    name: id.to_string(),
                    cred_def_id: String::from("cred_def_id"),
                    mime_type: String::from("application/json"),
                    value: String::from("YmFzZSA2NCBkYXRhIHN0cmluZw"),
                    referent: String::from("referent"),
                }]
                .to_vec(),
            ),

            predicate: Some(
                [Predicate {
                    name: String::from("some name"),
                    cred_def_id: String::from("cred_def_id"),
                    predicate: String::from("application/json"),
                    threshold: 5,
                }]
                .to_vec(),
            ),
        }),
        comment: None,
        presentation_attach: None,
    };

    let exchange_response = format!(
        r#"{{
            "type": "{}/propose-presentation",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        PRESENT_PROOF_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&presentation_data)?,
        id
    );
    let results = vade.didcomm_send(&options, &exchange_response).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<EncryptedMessage> = serde_json::from_str(result)?;

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_presentation_proposal(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(&options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<MessageWithBody<PresentationData>> =
        serde_json::from_str(result)?;

    let received_proposal = received
        .message
        .body
        .ok_or("send DIDComm request does not return presentation request".to_owned())?;

    let state = received_proposal.state;

    let proposal_data = received_proposal
        .presentation_proposal
        .ok_or("Proposal data not attached")?;
    let attribute_data = proposal_data
        .attribute
        .ok_or("Attributes not provided with proposal")?;
    let attribute = attribute_data.get(0).ok_or("Attribute is invalid")?;

    let proposal_data_saved = get_presentation(sender, receiver, id, state)?.presentation_proposal;
    let proposal_data_saved_attributes =
        proposal_data_saved.ok_or("Proposal data not saved in db")?;
    let attribute_data_saved = proposal_data_saved_attributes
        .attribute
        .ok_or("Attributes not saved in db")?;
    let attribute_saved = attribute_data_saved
        .get(0)
        .ok_or("Saved Attribute is invalid")?;
    assert_eq!(attribute.value, attribute_saved.value);
    return Ok(());
}

async fn send_ack(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let ack = Ack {
        r#type: String::from("https://didcomm.org/notification/1.0/ack"),
        from: Some(sender.to_string()),
        to: Some([receiver.to_string()].to_vec()),
        id: id.to_string(),
        thid: Some(id.to_string()),
        status: String::from("Success"),
        user_type: UserType::Verifier,
    };

    let exchange_complete = format!(
        r#"{{
            "type": "{}/ack",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        PRESENT_PROOF_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&ack)?,
        id
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
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive("{}", &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<Ack> = serde_json::from_str(result)?;

    let received_ack = received.message;

    assert_eq!(received_ack.thid.ok_or("Thread id not sent")?, id);

    return Ok(());
}

async fn send_problem_report(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let problem = ProblemReport {
        r#type: String::from("https://didcomm.org/report-problem/1.0/problem-report"),
        from: Some(sender.to_string()),
        to: Some([receiver.to_string()].to_vec()),
        id: id.to_string(),
        thid: Some(id.to_string()),
        description: Some(String::from("Request Rejected.")),
        problem_items: None,
        who_retries: None,
        fix_hint: None,
        impact: None,
        r#where: None,
        noticed_time: None,
        tracking_uri: None,
        excalation_uri: None,
        user_type: UserType::Prover,
    };

    let exchange_message = format!(
        r#"{{
            "type": "{}/problem-report",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        PRESENT_PROOF_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&problem)?,
        id
    );
    let results = vade.didcomm_send("{}", &exchange_message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<EncryptedMessage> = serde_json::from_str(result)?;

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_problem_report(
    vade: &mut Vade,
    _sender: &str,
    _receiver: &str,
    message: String,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive("{}", &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<ProblemReport> = serde_json::from_str(result)?;

    let received_problem = received.message;

    assert_eq!(received_problem.thid.ok_or("Thread id not sent")?, id);

    return Ok(());
}

#[tokio::test]
#[serial]
async fn can_do_presentation_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let user_1_did = String::from("did:uknow:d34db33d");
    let user_2_did = String::from("did:uknow:d34db33f");
    let options = String::from("{}");
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message =
        send_request_presentation(&mut vade, &user_1_did, &user_2_did, &options, &id).await?;
    receive_request_presentation(
        &mut vade,
        &user_1_did,
        &user_2_did,
        request_message,
        &options,
        &id,
    )
    .await?;

    let response_message =
        send_presentation(&mut vade, &user_2_did, &user_1_did, &options, &id).await?;
    receive_presentation(
        &mut vade,
        &user_2_did,
        &user_1_did,
        response_message,
        &options,
        &id,
    )
    .await?;

    let complete_message = send_ack(&mut vade, &user_1_did, &user_2_did, &id).await?;
    receive_ack(&mut vade, &user_1_did, &user_2_did, complete_message, &id).await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_proposal_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let user_1_did = String::from("did:uknow:d34db33d");
    let user_2_did = String::from("did:uknow:d34db33f");
    let options = String::from("{}");
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message =
        send_request_presentation(&mut vade, &user_1_did, &user_2_did, &options, &id).await?;
    receive_request_presentation(
        &mut vade,
        &user_1_did,
        &user_2_did,
        request_message,
        &options,
        &id,
    )
    .await?;

    let response_message =
        send_presentation_proposal(&mut vade, &user_2_did, &user_1_did, &options, &id).await?;
    receive_presentation_proposal(
        &mut vade,
        &user_2_did,
        &user_1_did,
        response_message,
        &options,
        &id,
    )
    .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_problem_report() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let user_1_did = String::from("did:uknow:d34db33d");
    let user_2_did = String::from("did:uknow:d34db33f");
    let options = String::from("{}");
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message =
        send_request_presentation(&mut vade, &user_1_did, &user_2_did, &options, &id).await?;
    receive_request_presentation(
        &mut vade,
        &user_1_did,
        &user_2_did,
        request_message,
        &options,
        &id,
    )
    .await?;

    let problem_message = send_problem_report(&mut vade, &user_1_did, &user_2_did, &id).await?;
    receive_problem_report(&mut vade, &user_1_did, &user_2_did, problem_message, &id).await?;

    Ok(())
}

async fn send_wrong_ack_state() -> Result<String, Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let user_1_did = String::from("did:uknow:d34db33d");
    let user_2_did = String::from("did:uknow:d34db33f");
    let options = String::from("{}");
    let id = Uuid::new_v4().to_simple().to_string();

    let _request_message =
        send_request_presentation(&mut vade, &user_1_did, &user_2_did, &options, &id).await?;

    let complete_message = send_ack(&mut vade, &user_1_did, &user_2_did, &id).await?;
    Ok(complete_message)
}

#[tokio::test]
#[should_panic]
#[serial]
async fn will_panic_and_fail_to_process_wrong_state() {
    let result = send_wrong_ack_state().await;
    match result {
        Err(e) => panic!("Error : {:?}", e),
        _ => {}
    }
}
