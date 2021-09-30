use didcomm_rs::Jwe;
use rocksdb::{DBWithThreadMode, SingleThreaded, DB};
use serial_test::serial;
use utilities::keypair::get_keypair_set;
use uuid::Uuid;
use vade::Vade;
use vade_didcomm::{
    datatypes::{MessageWithBody, VadeDidCommPluginOutput},
    protocols::issue_credential::datatypes::{
        Ack,
        Attribute,
        CredentialAttach,
        CredentialData,
        CredentialPreview,
        CredentialProposal,
        ProblemReport,
        State,
        UserType,
        ISSUE_CREDENTIAL_PROTOCOL_URL,
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

pub fn get_credential(
    from_did: &str,
    to_did: &str,
    thid: &str,
    state: State,
) -> Result<CredentialData, Box<dyn std::error::Error>> {
    let credential = read_db(&format!(
        "issue_credential_{}_{}_{}_{}",
        from_did, to_did, state, thid
    ))?;
    let credential_data: CredentialData = serde_json::from_str(&credential)?;
    Ok(credential_data)
}

async fn get_vade() -> Result<Vade, Box<dyn std::error::Error>> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new()?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

async fn send_propose_credential(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let credential_data = CredentialData {
        state: State::SendProposeCredential,
        credential_proposal: Some(CredentialProposal {
            id: id.to_string(),
            comment: String::from("No comment"),
            schema_issuer_did: sender.to_string(),
            schema_id: String::from("some_id"),
            schema_name: String::from("name"),
            schema_version: String::from("version"),
            cred_def_id: String::from("cred_id"),
            issuer_did: String::from("issuer_did"),
        }),
        credential_preview: Some(CredentialPreview {
            r#type: String::from(""),
            attributes: [Attribute {
                name: String::from("atr-name"),
                mime_type: String::from("text"),
                value: String::from("abc_123"),
            }]
            .to_vec(),
        }),
        data_attach: None,
        comment: None,
    };

    let exchange_request = format!(
        r#"{{
            "type": "{}/propose-credential",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        ISSUE_CREDENTIAL_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&credential_data)?,
        id
    );

    let results = vade.didcomm_send(options, &exchange_request).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let prepared: VadeDidCommPluginOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_propose_credential(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let received: VadeDidCommPluginOutput<MessageWithBody<CredentialData>> =
        serde_json::from_str(result)?;

    let propose_credential = received
        .message
        .body
        .ok_or("send DIDComm request does not return propose credential".to_owned())?;

    let attached_req = propose_credential
        .credential_proposal
        .ok_or("Proposal not attached")?;

    let req_data_saved = get_credential(sender, receiver, id, propose_credential.state)?;
    let attached_req_saved = req_data_saved
        .credential_proposal
        .ok_or("Proposal data not attached")?;

    assert_eq!(attached_req.id, attached_req_saved.id);

    Ok(())
}

async fn send_offer_credential(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let credential_data = CredentialData {
        state: State::SendOfferCredential,
        credential_proposal: None,
        credential_preview: Some(CredentialPreview {
            r#type: String::from(""),
            attributes: [Attribute {
                name: String::from("atr-name"),
                mime_type: String::from("text"),
                value: String::from("abc_123"),
            }]
            .to_vec(),
        }),
        data_attach: Some(
            [CredentialAttach {
                id: String::from("id"),
                mime_type: String::from("text"),
                data: String::from("YmFzZSA2NCBkYXRhIHN0cmluZw"),
            }]
            .to_vec(),
        ),
        comment: None,
    };

    let exchange_response = format!(
        r#"{{
            "type": "{}/offer-credential",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        ISSUE_CREDENTIAL_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&credential_data)?,
        id
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

async fn receive_offer_credential(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<MessageWithBody<CredentialData>> =
        serde_json::from_str(result)?;

    let received_offer = received
        .message
        .body
        .ok_or("send DIDComm request does not return offer credential request".to_owned())?;

    let state = received_offer.state;
    let attached_data = received_offer
        .data_attach
        .ok_or("Offer credential request not attached")?;
    let credential_data = attached_data.get(0).ok_or("Request body is invalid")?;

    let req_data_saved = get_credential(sender, receiver, id, state)?;
    let attached_credential_saved = req_data_saved
        .data_attach
        .ok_or("Offer Credential request not saved in DB")?;
    let attached_data_saved = attached_credential_saved
        .get(0)
        .ok_or("Request body is invalid")?;

    assert_eq!(credential_data.data, attached_data_saved.data);

    Ok(())
}

async fn send_request_credential(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let credential_data = CredentialData {
        state: State::SendRequestCredential,
        credential_proposal: None,
        credential_preview: None,
        data_attach: Some(
            [CredentialAttach {
                id: String::from("id"),
                mime_type: String::from("text"),
                data: String::from("YmFzZSA2NCBkYXRhIHN0cmluZw"),
            }]
            .to_vec(),
        ),
        comment: None,
    };

    let exchange_response = format!(
        r#"{{
            "type": "{}/request-credential",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        ISSUE_CREDENTIAL_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&credential_data)?,
        id
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

async fn receive_request_credential(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<MessageWithBody<CredentialData>> =
        serde_json::from_str(result)?;

    let received_proposal = received
        .message
        .body
        .ok_or("send DIDComm request does not return request credential request".to_owned())?;

    let state = received_proposal.state;

    let proposal_data = received_proposal
        .data_attach
        .ok_or("Request crendential data not attached")?;

    let attribute = proposal_data.get(0).ok_or("Attachment is invalid")?;

    let proposal_data_saved = get_credential(sender, receiver, id, state)?.data_attach;
    let proposal_data_saved_attributes =
        proposal_data_saved.ok_or("Request crendential data not saved in db")?;

    let attribute_saved = proposal_data_saved_attributes
        .get(0)
        .ok_or("Saved Attachment is invalid")?;
    assert_eq!(attribute.data, attribute_saved.data);
    Ok(())
}

async fn send_issue_credential(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let credential_data = CredentialData {
        state: State::SendIssueCredential,
        credential_proposal: None,
        credential_preview: None,
        data_attach: Some(
            [CredentialAttach {
                id: String::from("id"),
                mime_type: String::from("text"),
                data: String::from("YmFzZSA2NCBkYXRhIHN0cmluZw"),
            }]
            .to_vec(),
        ),
        comment: None,
    };

    let exchange_response = format!(
        r#"{{
            "type": "{}/issue-credential",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        ISSUE_CREDENTIAL_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&credential_data)?,
        id
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

async fn receive_issue_credential(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
    options: &str,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = vade.didcomm_receive(options, &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    let received: VadeDidCommPluginOutput<MessageWithBody<CredentialData>> =
        serde_json::from_str(result)?;

    let received_proposal = received
        .message
        .body
        .ok_or("send DIDComm request does not return issue credential request".to_owned())?;

    let state = received_proposal.state;

    let proposal_data = received_proposal
        .data_attach
        .ok_or("Issue Credential data not attached")?;

    let attachment = proposal_data.get(0).ok_or("Attachment is invalid")?;

    let proposal_data_saved = get_credential(sender, receiver, id, state)?.data_attach;
    let proposal_data_saved_attributes =
        proposal_data_saved.ok_or("Issue Credential not saved in db")?;

    let attachment_saved = proposal_data_saved_attributes
        .get(0)
        .ok_or("Saved Attachment is invalid")?;
    assert_eq!(attachment.data, attachment_saved.data);
    Ok(())
}

async fn send_ack(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let ack = Ack {
        r#type: String::from("https://didcomm.org/notification/1.0/ack"),
        from: Some(sender.to_string()),
        to: Some([receiver.to_string()].to_vec()),
        id: id.to_string(),
        thid: Some(id.to_string()),
        status: String::from("Success"),
        user_type: UserType::Holder,
    };

    let exchange_complete = format!(
        r#"{{
            "type": "{}/ack",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        ISSUE_CREDENTIAL_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&ack)?,
        id
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

async fn receive_ack(
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

    let received: VadeDidCommPluginOutput<Ack> = serde_json::from_str(result)?;

    let received_ack = received.message;

    assert_eq!(received_ack.thid.ok_or("Thread id not sent")?, id);

    Ok(())
}

async fn send_problem_report(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
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
        user_type: UserType::Issuer,
    };

    let exchange_message = format!(
        r#"{{
            "type": "{}/problem-report",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "thid": "{}"
        }}"#,
        ISSUE_CREDENTIAL_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&problem)?,
        id
    );
    let results = vade.didcomm_send(options, &exchange_message).await?;
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

    let received: VadeDidCommPluginOutput<ProblemReport> = serde_json::from_str(result)?;

    let received_problem = received.message;

    assert_eq!(received_problem.thid.ok_or("Thread id not sent")?, id);

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_issue_credential() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message = send_propose_credential(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_propose_credential(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        request_message,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;

    let response_message = send_offer_credential(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;
    receive_offer_credential(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        response_message,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;

    let offer_credential = send_request_credential(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_request_credential(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        offer_credential,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;

    let issue_credential = send_issue_credential(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;
    receive_issue_credential(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        issue_credential,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;

    let complete_message = send_ack(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_ack(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.receiver_options_stringified,
        complete_message,
        &id,
    )
    .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_problem_report() -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message = send_propose_credential(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_propose_credential(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        request_message,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;

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

async fn send_wrong_ack_state() -> Result<String, Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let id = Uuid::new_v4().to_simple().to_string();

    let _request_message = send_propose_credential(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;

    let complete_message = send_ack(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
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
