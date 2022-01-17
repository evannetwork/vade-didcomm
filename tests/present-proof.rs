mod common;

use std::collections::HashMap;

use common::{get_vade, read_db};
use didcomm_rs::Jwe;
use serial_test::serial;
use utilities::keypair::get_keypair_set;
use uuid::Uuid;
use vade::Vade;
use vade_didcomm::{
    datatypes::{MessageWithBody, VadeDidCommPluginOutput},
    protocols::present_proof::datatypes::{
        AckData,
        AckStatus,
        Attribute,
        MessageData,
        Predicate,
        PresentationAttach,
        PresentationData,
        PresentationPreview,
        ProblemReportData,
        ProposalData,
        RequestData,
        State,
        UserType,
        PRESENT_PROOF_PROTOCOL_URL,
    },
};

pub fn create_new_message<T: MessageData>(
    from: &str,
    to: &str,
    thid: &str,
    type_suffix: &str,
    body: T,
) -> Result<MessageWithBody<T>, Box<dyn std::error::Error>> {
    Ok(MessageWithBody::<T> {
        r#type: format!("{}/{}", PRESENT_PROOF_PROTOCOL_URL, &type_suffix),
        from: Some(from.to_string()),
        to: Some(vec![to.to_string()]),
        thid: Some(thid.to_string()),
        body: Some(body),
        created_time: None,
        expires_time: None,
        id: None,
        pthid: None,
        other: HashMap::new(),
    })
}

pub fn get_presentation_data<T: MessageData + serde::de::DeserializeOwned>(
    from_did: &str,
    to_did: &str,
    thid: &str,
    state: State,
) -> Result<T, Box<dyn std::error::Error>> {
    let presentation = read_db(&format!(
        "present_proof_{}_{}_{}_{}",
        from_did, to_did, state, thid
    ))?;
    let presentation_data: T = serde_json::from_str(&presentation)?;
    Ok(presentation_data)
}

async fn send_request_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    thid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_request = create_new_message(
        sender,
        receiver,
        thid,
        "request-presentation",
        RequestData {
            request_presentations_attach: [PresentationAttach {
                id: Uuid::new_v4().to_simple().to_string(),
                mime_type: String::from("application/json"),
                data: String::from("YmFzZSA2NCBkYXRhIHN0cmluZw"),
            }]
            .to_vec(),
            comment: None,
        },
    )?;

    let results = vade
        .didcomm_send(options, &serde_json::to_string(&exchange_request)?)
        .await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_request_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    thid: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let received: VadeDidCommPluginOutput<MessageWithBody<RequestData>> =
        serde_json::from_str(result)?;

    let request_presentation = received
        .message
        .body
        .ok_or_else(|| "send DIDComm request does not return presentation request".to_string())?;

    let attached_req = request_presentation.request_presentations_attach;
    let presentation_data = attached_req.get(0).ok_or("Request body is invalid")?;
    let req_data_saved: RequestData =
        get_presentation_data(sender, receiver, thid, State::PresentationRequested)?;
    let attached_req_saved = req_data_saved.request_presentations_attach;
    let presentation_data_saved = attached_req_saved.get(0).ok_or("Request body is invalid")?;

    assert_eq!(presentation_data.data, presentation_data_saved.data);

    Ok(())
}

async fn send_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    thid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let exchange_response = create_new_message(
        sender,
        receiver,
        thid,
        "presentation",
        PresentationData {
            presentations_attach: [PresentationAttach {
                id: Uuid::new_v4().to_simple().to_string(),
                mime_type: String::from("application/json"),
                data: String::from("YmFzZSA2NCBkYXRhIHN0cmluZw"),
            }]
            .to_vec(),

            comment: None,
        },
    )?;

    let results = vade
        .didcomm_send(options, &serde_json::to_string(&exchange_response)?)
        .await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    thid: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
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
        .ok_or("send DIDComm request does not return presentation".to_string())?;

    let attached_presentation = received_presentation.presentations_attach;
    let presentation_data = attached_presentation
        .get(0)
        .ok_or("Presentation body is invalid")?;

    let req_data_saved: PresentationData =
        get_presentation_data(sender, receiver, thid, State::PresentationSent)?;
    let attached_presentation_saved = req_data_saved.presentations_attach;
    let presentation_data_saved = attached_presentation_saved
        .get(0)
        .ok_or("Presentation body is invalid")?;

    assert_eq!(presentation_data.data, presentation_data_saved.data);

    Ok(())
}

async fn send_presentation_proposal(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    thid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let proposal = create_new_message(
        sender,
        receiver,
        thid,
        "propose-presentation",
        ProposalData {
            presentation_proposal: PresentationPreview {
                r#type: format!("{}/presentation-preview", PRESENT_PROOF_PROTOCOL_URL),

                attribute: Some(
                    [Attribute {
                        name: thid.to_string(),
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
            },
            comment: None,
        },
    )?;

    let results = vade
        .didcomm_send(options, &serde_json::to_string(&proposal)?)
        .await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_presentation_proposal(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    thid: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<MessageWithBody<ProposalData>> =
        serde_json::from_str(result)?;

    let received_proposal = received
        .message
        .body
        .ok_or_else(|| "send DIDComm request does not return presentation request".to_string())?;

    let attribute_data = received_proposal
        .presentation_proposal
        .attribute
        .ok_or("Attributes not provided with proposal")?;
    let attribute = attribute_data.get(0).ok_or("Attribute is invalid")?;

    let proposal_data_saved: ProposalData =
        get_presentation_data(sender, receiver, thid, State::PresentationProposed)?;
    let attribute_data_saved = proposal_data_saved
        .presentation_proposal
        .attribute
        .ok_or("Attributes not saved in db")?;
    let attribute_saved = attribute_data_saved
        .get(0)
        .ok_or("Saved Attribute is invalid")?;
    assert_eq!(attribute.value, attribute_saved.value);
    Ok(())
}

async fn send_ack(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    thid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let ack = create_new_message(
        sender,
        receiver,
        thid,
        "ack",
        AckData {
            status: AckStatus::OK,
        },
    )?;

    let results = vade
        .didcomm_send(options, &serde_json::to_string(&ack)?)
        .await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_ack(
    vade: &mut Vade,
    _sender: &str,
    _receiver: &str,
    message: String,
    options: &str,
    thid: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<MessageWithBody<AckData>> = serde_json::from_str(result)?;

    let received_ack = received.message;

    assert_eq!(received_ack.thid.ok_or("Thread id not sent")?, thid);

    Ok(())
}

async fn send_problem_report(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    thid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let problem_report = create_new_message(
        sender,
        receiver,
        thid,
        "problem-report",
        ProblemReportData {
            description: Some(String::from("Request Rejected.")),
            problem_items: None,
            who_retries: None,
            fix_hint: None,
            impact: None,
            r#where: None,
            noticed_time: None,
            tracking_uri: None,
            escalation_uri: None,
            user_type: UserType::Prover,
        },
    )?;

    let results = vade
        .didcomm_send(options, &serde_json::to_string(&problem_report)?)
        .await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_problem_report(
    vade: &mut Vade,
    _sender: &str,
    _receiver: &str,
    message: String,
    options: &str,
    thid: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<MessageWithBody<ProblemReportData>> =
        serde_json::from_str(result)?;

    let received_problem = received.message;

    assert_eq!(received_problem.thid.ok_or("Thread id not sent")?, thid);

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_presentation_exchange_for_present_proof() -> Result<(), Box<dyn std::error::Error>>
{
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let thid = Uuid::new_v4().to_simple().to_string();

    let request_message = send_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;
    receive_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        request_message,
        &test_setup.receiver_options_stringified,
        &thid,
    )
    .await?;

    let response_message = send_presentation(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.receiver_options_stringified,
        &thid,
    )
    .await?;
    receive_presentation(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        response_message,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;

    let complete_message = send_ack(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;
    receive_ack(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        complete_message,
        &test_setup.receiver_options_stringified,
        &thid,
    )
    .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_proposal_exchange_for_present_proof() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let thid = Uuid::new_v4().to_simple().to_string();

    let request_message = send_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;
    receive_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        request_message,
        &test_setup.receiver_options_stringified,
        &thid,
    )
    .await?;

    let response_message = send_presentation_proposal(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.receiver_options_stringified,
        &thid,
    )
    .await?;
    receive_presentation_proposal(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        response_message,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_problem_report() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let thid = Uuid::new_v4().to_simple().to_string();

    let request_message = send_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;
    receive_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        request_message,
        &test_setup.receiver_options_stringified,
        &thid,
    )
    .await?;

    let problem_message = send_problem_report(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;
    receive_problem_report(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        problem_message,
        &test_setup.receiver_options_stringified,
        &thid,
    )
    .await?;

    Ok(())
}

async fn send_wrong_ack_state() -> Result<String, Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let thid = Uuid::new_v4().to_simple().to_string();

    let _request_message = send_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;

    let complete_message = send_ack(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &thid,
    )
    .await?;
    Ok(complete_message)
}

#[tokio::test]
#[should_panic]
#[serial]
async fn will_panic_and_fail_to_process_wrong_state() {
    let result = send_wrong_ack_state().await;
    if let Err(e) = result {
        panic!("Error : {:?}", e)
    }
}
