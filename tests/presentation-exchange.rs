mod common;

extern crate jsonpath_lib as jsonpath;

use std::cmp::Ordering;

use common::{get_vade, read_db};
use didcomm_rs::Jwe;
use serial_test::serial;
use utilities::keypair::get_keypair_set;
use uuid::Uuid;
use vade::Vade;
use vade_didcomm::{
    datatypes::{
        ExtendedMessage,
        MessageWithBody,
        VadeDidCommPluginReceiveOutput,
        VadeDidCommPluginSendOutput,
    },
    protocols::presentation_exchange::datatypes::{
        Attachment,
        Constraints,
        CredentialSubject,
        Data,
        DescriptorMap,
        Field,
        Format,
        InputDescriptor,
        JsonData,
        Options,
        PresentationDefinition,
        PresentationExchangeData,
        PresentationSubmission,
        Proof,
        ProofType,
        Schema,
        State,
        VerifiableCredential,
        PRESENTATION_EXCHANGE_PROTOCOL_URL,
    },
};

pub fn get_presentation_exchange(
    from_did: &str,
    to_did: &str,
    thid: &str,
    state: State,
) -> Result<PresentationExchangeData, Box<dyn std::error::Error>> {
    let presentation = read_db(&format!(
        "presentation_exchange_{}_{}_{}_{}",
        from_did, to_did, state, thid
    ))?;
    let presentation_data: PresentationExchangeData = serde_json::from_str(&presentation)?;
    Ok(presentation_data)
}

async fn send_request_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation_data = PresentationExchangeData {
        state: State::SendPresentationRequest,
        format: Some(
            [Format {
                attach_id: Some(id.to_string()),
                format: Some(String::from("dif/presentation-exchange/definition@v1.0")),
                jwt: None,
                jwt_vc: None,
                jwt_vp: None,
                ldp_vc: None,
                ldp_vp: None,
                ldp: None,
            }]
            .to_vec(),
        ),
        request_presentation_attach: Some(Attachment {
            id: id.to_string(),
            mime_type: String::from("application/json"),
            data: [Data {
                json: JsonData {
                    presentation_definition: Some(PresentationDefinition {
                        id: id.to_string(),
                        input_descriptors: [InputDescriptor {
                            id: "citizenship_input".to_string(),
                            name: Some("US Passport".to_string()),
                            format: None,
                            schema: [Schema {
                                uri: "hub://did:foo:123/Collections/schema.us.gov/passport.json"
                                    .to_string(),
                            }]
                            .to_vec(),
                            group: Some(["A".to_string()].to_vec()),
                            purpose: None,
                            constraints: Some(Constraints {
                                fields: [Field {
                                    path: [
                                        "$.credentialSubject.birth_date".to_string(),
                                        "$.birth_date".to_string(),
                                    ]
                                    .to_vec(),
                                    id: Some("Some_field_id".to_string()),
                                    purpose: None,
                                    predicate: None,
                                    filter: Some(
                                        [
                                            ("type".to_string(), "date".to_string()),
                                            ("minimum".to_string(), "1978-05-16".to_string()),
                                        ]
                                        .iter()
                                        .cloned()
                                        .collect(),
                                    ),
                                }]
                                .to_vec(),
                                limit_disclosure: None,
                                statuses: None,
                                subject_is_issuer: None,
                                is_holder: None,
                                same_subject: None,
                            }),
                        }]
                        .to_vec(),
                        format: Some(Format {
                            attach_id: None,
                            format: None,
                            jwt: None,
                            jwt_vc: None,
                            jwt_vp: None,
                            ldp_vc: None,
                            ldp_vp: Some(ProofType {
                                proof_type: ["Ed25519Signature2018".to_string()].to_vec(),
                            }),
                            ldp: None,
                        }),
                        name: None,
                        purpose: None,
                        submission_requirements: None,
                    }),
                    options: Some(Options {
                        challenge: "23516943-1d79-4ebd-8981-623f036365ef".to_string(),
                        domain: "us.gov/DriversLicense".to_string(),
                    }),
                    input_descriptors: None,
                    context: None,
                    r#type: None,
                    presentation_submission: None,
                    verifiable_credential: None,
                    proof: None,
                },
            }]
            .to_vec(),
        }),
        comment: None,
        proposal_attach: None,
        presentations_attach: None,
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
        PRESENTATION_EXCHANGE_PROTOCOL_URL,
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

    let prepared: VadeDidCommPluginSendOutput<Jwe> = serde_json::from_str(result)?;

    Ok(serde_json::to_string(&prepared.message)?)
}

async fn receive_request_presentation(
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
    let received: VadeDidCommPluginReceiveOutput<MessageWithBody<PresentationExchangeData>> =
        serde_json::from_str(result)?;

    let request_presentation = received
        .message
        .body
        .ok_or_else(|| "send DIDComm request does not return presentation request".to_string())?;

    let attached_req = request_presentation
        .request_presentation_attach
        .ok_or("Presentation request not attached")?;

    let presentation_data = &attached_req
        .data
        .get(0)
        .ok_or("Data not found")?
        .json
        .presentation_definition
        .as_ref()
        .ok_or("presentation definition not found")?
        .input_descriptors
        .get(0)
        .ok_or("input descriptor not found")?
        .name;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let req_data_saved =
                get_presentation_exchange(sender, receiver, id, request_presentation.state)?;

            let attached_req_saved = req_data_saved
                .request_presentation_attach
                .ok_or("Presentation request not attached")?;

            let presentation_data_saved = &attached_req_saved
                .data
                .get(0)
                .ok_or("Data not found")?
                .json
                .presentation_definition
                .as_ref()
                .ok_or("presentation definition not found")?
                .input_descriptors
                .get(0)
                .ok_or("input descriptor not found")?
                .name;

            assert_eq!(presentation_data, presentation_data_saved);
        } else {}
    }

    Ok(())
}

async fn send_presentation(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation_data = PresentationExchangeData {
        state: State::SendPresentation,
        format: Some(
            [Format {
                attach_id: Some(id.to_string()),
                format: Some(String::from("dif/presentation-exchange/definition@v1.0")),
                jwt: None,
                jwt_vc: None,
                jwt_vp: None,
                ldp_vc: None,
                ldp_vp: None,
                ldp: None,
            }]
            .to_vec(),
        ),
        presentations_attach: Some(Attachment {
            id: id.to_string(),
            mime_type: String::from("application/json"),
            data: [Data {
                json: JsonData {
                    presentation_definition: None,
                    options: None,
                    input_descriptors: None,
                    context: Some(
                        [
                            "https://www.w3.org/2018/credentials/v1".to_string(),
                            "https://identity.foundation/presentation-exchange/submission/v1"
                                .to_string(),
                        ]
                        .to_vec(),
                    ),
                    r#type: Some(
                        [
                            "VerifiablePresentation".to_string(),
                            "PresentationSubmission".to_string(),
                        ]
                        .to_vec(),
                    ),
                    presentation_submission: Some(PresentationSubmission {
                        id: None,
                        definition_id: None,
                        descriptor_map: Some(
                            [DescriptorMap {
                                id: "citizenship_input".to_string(),
                                path: "$.verifiableCredential.[0]".to_string(),
                                format: None,
                                path_nested: None,
                            }]
                            .to_vec(),
                        ),
                    }),
                    verifiable_credential: Some(
                        [VerifiableCredential {
                            context: "https://www.w3.org/2018/credentials/v1".to_string(),
                            id: "https://eu.com/claims/DriversLicense".to_string(),
                            r#type: "US Passport".to_string(),
                            issuer: "did:foo:123".to_string(),
                            issuance_date: "2010-01-01T19:73:24Z".to_string(),
                            credential_subject: CredentialSubject {
                                id: "did:example:ebfeb1f712ebc6f1c276e12ec21".to_string(),
                                data: [
                                    ("type".to_string(), "passport".to_string()),
                                    ("number".to_string(), "34DGE352".to_string()),
                                    ("dob".to_string(), "1980-07-09".to_string()),
                                ]
                                .iter()
                                .cloned()
                                .collect(),
                            },
                            proof: Proof {
                                r#type: "RsaSignature2018".to_string(),
                                created: "2017-06-18T21:19:10Z".to_string(),
                                proof_purpose: "assertionMethod".to_string(),
                                verification_method: "https://example.edu/issuers/keys/1"
                                    .to_string(),
                                jws: "...".to_string(),
                                challenge: None,
                                domain: None,
                            },
                        }]
                        .to_vec(),
                    ),
                    proof: Some(Proof {
                        r#type: "RsaSignature2018".to_string(),
                        created: "2017-06-18T21:19:10Z".to_string(),
                        proof_purpose: "assertionMethod".to_string(),
                        verification_method: "https://example.edu/issuers/keys/1".to_string(),
                        jws: "...".to_string(),
                        challenge: Some("1f44d55f-f161-4938-a659-f8026467f126".to_string()),
                        domain: Some("4jt78h47fh47".to_string()),
                    }),
                },
            }]
            .to_vec(),
        }),
        comment: None,
        proposal_attach: None,
        request_presentation_attach: None,
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
        PRESENTATION_EXCHANGE_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&presentation_data)?,
        id
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

async fn receive_presentation(
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

    let received: VadeDidCommPluginReceiveOutput<MessageWithBody<PresentationExchangeData>> =
        serde_json::from_str(result)?;

    let received_presentation = received
        .message
        .body
        .ok_or_else(|| "send DIDComm request does not return presentation request".to_string())?;

    let state = received_presentation.state;
    let attached_presentation = received_presentation
        .presentations_attach
        .ok_or("Presentation not attached")?;

    let data = &attached_presentation
        .data
        .get(0)
        .ok_or("Presentation data is empty")?
        .json
        .verifiable_credential
        .as_ref()
        .ok_or("Credentials not attached")?
        .get(0)
        .ok_or("Credentials are empty")?
        .r#type;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let req_data_saved = get_presentation_exchange(sender, receiver, id, state)?;
            let req_data_saved_cloned = req_data_saved.clone();

            let attached_presentation_saved = req_data_saved
                .presentations_attach
                .ok_or("Presentation not attached")?;

            let data_saved = &attached_presentation_saved
                .data
                .get(0)
                .ok_or("Presentation data is empty")?
                .json
                .verifiable_credential
                .as_ref()
                .ok_or("Credentials not attached")?
                .get(0)
                .ok_or("Credentials are empty")?
                .r#type;

            assert_eq!(data, data_saved);

            let sent_request =
                get_presentation_exchange(receiver, sender, id, State::SendPresentationRequest)?;

            let requested_json = serde_json::to_value(sent_request)?;
            let mut requested_selector = jsonpath::selector(&requested_json);

            let received_json = serde_json::to_value(req_data_saved_cloned)?;
            let mut received_selector = jsonpath::selector(&received_json);

            let passport_dob = received_selector(
                "$.presentations_attach.data[0].json.verifiable_credential[*].credentialSubject.data.dob",
            )?
            .get(0)
            .ok_or("Dob not provided")?
            .to_string();

            let minimum_date = requested_selector(
                "$.request_presentation_attach.data[0].json.presentation_definition.input_descriptors[*].constraints.fields[*].filter.minimum",
            )?
            .get(0)
            .ok_or("Date filter not provided")?
            .to_string();

            assert_eq!(passport_dob.cmp(&minimum_date), Ordering::Greater);
        } else {}
    }

    Ok(())
}

async fn send_presentation_proposal(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    options: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation_data = PresentationExchangeData {
        state: State::SendProposePresentation,
        format: Some(
            [Format {
                attach_id: Some(id.to_string()),
                format: Some(String::from("dif/presentation-exchange/definition@v1.0")),
                jwt: None,
                jwt_vc: None,
                jwt_vp: None,
                ldp_vc: None,
                ldp_vp: None,
                ldp: None,
            }]
            .to_vec(),
        ),
        proposal_attach: Some(Attachment {
            id: id.to_string(),
            mime_type: String::from("application/json"),
            data: [Data {
                json: JsonData {
                    input_descriptors: Some(
                        [InputDescriptor {
                            id: "citizenship_input".to_string(),
                            name: Some("US Passport".to_string()),
                            format: None,
                            schema: [Schema {
                                uri: "hub://did:foo:123/Collections/schema.us.gov/passport.json"
                                    .to_string(),
                            }]
                            .to_vec(),
                            group: Some(["A".to_string()].to_vec()),
                            purpose: None,
                            constraints: Some(Constraints {
                                fields: [Field {
                                    path: [
                                        "$.credentialSubject.birth_date".to_string(),
                                        "$.birth_date".to_string(),
                                    ]
                                    .to_vec(),
                                    id: Some("Some_field_id".to_string()),
                                    purpose: None,
                                    predicate: None,
                                    filter: Some(
                                        [
                                            ("type".to_string(), "date".to_string()),
                                            ("minimum".to_string(), "1999-5-16".to_string()),
                                        ]
                                        .iter()
                                        .cloned()
                                        .collect(),
                                    ),
                                }]
                                .to_vec(),
                                limit_disclosure: None,
                                statuses: None,
                                subject_is_issuer: None,
                                is_holder: None,
                                same_subject: None,
                            }),
                        }]
                        .to_vec(),
                    ),
                    options: None,
                    context: None,
                    r#type: None,
                    presentation_submission: None,
                    verifiable_credential: None,
                    proof: None,
                    presentation_definition: None,
                },
            }]
            .to_vec(),
        }),
        comment: None,
        presentations_attach: None,
        request_presentation_attach: None,
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
        PRESENTATION_EXCHANGE_PROTOCOL_URL,
        sender,
        receiver,
        &serde_json::to_string(&presentation_data)?,
        id
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

async fn receive_presentation_proposal(
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

    let received: VadeDidCommPluginReceiveOutput<MessageWithBody<PresentationExchangeData>> =
        serde_json::from_str(result)?;

    let received_proposal = received
        .message
        .body
        .ok_or_else(|| "send DIDComm request does not return presentation request".to_string())?;

    let state = received_proposal.state;

    let proposal_data = received_proposal
        .proposal_attach
        .ok_or("Proposal data not attached")?;

    let attribute_data = &proposal_data
        .data
        .get(0)
        .ok_or("Data not found")?
        .json
        .input_descriptors
        .as_ref()
        .ok_or("Proposal data not found")?
        .get(0)
        .ok_or("input descriptor not found")?
        .name;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let proposal_data_saved =
                get_presentation_exchange(sender, receiver, id, state)?.proposal_attach;
            let proposal_data_saved_attributes =
                proposal_data_saved.ok_or("Proposal data not saved in db")?;

            let attribute_data_saved = &proposal_data_saved_attributes
                .data
                .get(0)
                .ok_or("Data not found")?
                .json
                .input_descriptors
                .as_ref()
                .ok_or("Proposal data not found")?
                .get(0)
                .ok_or("input descriptor not found")?
                .name;

            assert_eq!(attribute_data, attribute_data_saved);
        } else {}
    }

    Ok(())
}

async fn get_messages_by_thid(thid: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let message_id = format!("message_{}_*", thid);

    let results = vade
        .run_custom_function("{}", "query_didcomm_messages", "{}", &message_id)
        .await?;
    let received = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    Ok(received.to_string())
}

async fn get_messages_by_msgid(
    thid: &str,
    msg_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let key = format!("message_{}_{}", thid, msg_id);

    let results = vade
        .run_custom_function("{}", "query_didcomm_messages", "{}", &key)
        .await?;
    let received = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;

    Ok(received.to_string())
}

#[tokio::test]
#[serial]
async fn can_do_presentation_exchange_for_presentation_exchange(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message = send_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        request_message,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;

    let response_message = send_presentation(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_presentation(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        response_message,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn can_do_proposal_exchange_for_presentation_exchange(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message = send_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        request_message,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;

    let response_message = send_presentation_proposal(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_presentation_proposal(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        response_message,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;

    Ok(())
}

#[tokio::test]
#[serial]
#[cfg(feature = "state_storage")]
async fn can_do_presentation_exchange_and_fetch_all_messages_from_didcomm_by_thid(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut vade = get_vade().await?;
    let test_setup = get_keypair_set();
    let id = Uuid::new_v4().to_simple().to_string();

    let request_message = send_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_request_presentation(
        &mut vade,
        &test_setup.user1_did,
        &test_setup.user2_did,
        request_message,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;

    let response_message = send_presentation(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        &test_setup.sender_options_stringified,
        &id,
    )
    .await?;
    receive_presentation(
        &mut vade,
        &test_setup.user2_did,
        &test_setup.user1_did,
        response_message,
        &test_setup.receiver_options_stringified,
        &id,
    )
    .await?;

    // Fetch all messages
    let messages = get_messages_by_thid(&id).await?;

    let messages: Vec<String> = serde_json::from_str(&messages)?;

    // Total 4 messages should be fetched
    assert!(
        messages.len() == 4,
        "Invalid message count for thid {} found in rocks db",
        id
    );

    // Get message sent in exchange at index 0
    let message_0 = messages.get(0).ok_or("Invalid message stored in didcomm")?;

    let parsed_message_0: ExtendedMessage = serde_json::from_str(&message_0)?;

    let message_id = parsed_message_0.id.ok_or("Invalid id")?;

    // Fetch message by thid and msgid
    let message = get_messages_by_msgid(&id, &message_id).await?;
    let message: Vec<String> = serde_json::from_str(&message)?;
    let message = message.get(0).ok_or("Invalid message stored in didcomm")?;

    let parsed_message_1: ExtendedMessage = serde_json::from_str(&message)?;

    // Message fetch by id and Message fetched by thid at index 0 should be equal
    assert_eq!(parsed_message_1.r#type, parsed_message_0.r#type);

    assert_eq!(parsed_message_1.created_time, parsed_message_0.created_time);

    assert_eq!(parsed_message_1.body, parsed_message_0.body);

    Ok(())
}
