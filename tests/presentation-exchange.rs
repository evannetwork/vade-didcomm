use rocksdb::{DBWithThreadMode, SingleThreaded, DB};
use serial_test::serial;
use uuid::Uuid;
use vade::Vade;
use vade_didcomm::{
    datatypes::{EncryptedMessage, MessageWithBody, VadeDidCommPluginOutput},
    protocols::presentation_exchange::datatypes::{
        Attachment, Constraints, CredentialSubject, Data, DescriptorMap, Field, Format,
        InputDescriptor, JsonData, Options, PresentationDefinition, PresentationExchangeData,
        PresentationSubmission, Proof, ProofType, Schema, State, VerifiableCredential,
        PRESENTATION_EXCHANGE_PROTOCOL_URL,
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
    thid: &str,
    state: State,
) -> Result<PresentationExchangeData, Box<dyn std::error::Error>> {
    let presentation = read_db(&format!(
        "presentation_exchange_{}_{}_{}_{}",
        from_did, to_did, state, thid
    ))?;
    let presentation_data: PresentationExchangeData = serde_json::from_str(&presentation)?;
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
                            name: "US Passport".to_string(),
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
                                    id: "Some_field_id".to_string(),
                                    purpose: None,
                                    predicate: None,
                                    filter: [
                                        ("type".to_string(), "date".to_string()),
                                        ("minimum".to_string(), "1999-5-16".to_string()),
                                    ]
                                    .iter()
                                    .cloned()
                                    .collect(),
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
    let received: VadeDidCommPluginOutput<MessageWithBody<PresentationExchangeData>> =
        serde_json::from_str(result)?;

    let request_presentation = received
        .message
        .body
        .ok_or("send DIDComm request does not return presentation request".to_owned())?;

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

    let req_data_saved = get_presentation(sender, receiver, id, request_presentation.state)?;

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

    return Ok(());
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
                            r#type: ["EUDriversLicense".to_string()].to_vec(),
                            issuer: "did:foo:123".to_string(),
                            issuance_date: "2010-01-01T19:73:24Z".to_string(),
                            credential_subject: CredentialSubject {
                                id: "did:example:ebfeb1f712ebc6f1c276e12ec21".to_string(),
                                data: [
                                    ("number".to_string(), "34DGE352".to_string()),
                                    ("dob".to_string(), "07/13/80".to_string()),
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

    let received: VadeDidCommPluginOutput<MessageWithBody<PresentationExchangeData>> =
        serde_json::from_str(result)?;

    let received_presentation = received
        .message
        .body
        .ok_or("send DIDComm request does not return presentation request".to_owned())?;

    let state = received_presentation.state;
    let attached_presentation = received_presentation
        .presentations_attach
        .ok_or("Presentation not attached")?;

    let data = attached_presentation
        .data
        .get(0)
        .ok_or("Presentation data is empty")?
        .json
        .verifiable_credential
        .as_ref()
        .ok_or("Credentials not attached")?
        .get(0)
        .ok_or("Credentials are empty")?
        .r#type
        .get(0)
        .ok_or("Credential type not found")?;

    let req_data_saved = get_presentation(sender, receiver, id, state)?;
    let attached_presentation_saved = req_data_saved
        .presentations_attach
        .ok_or("Presentation not attached")?;

    let data_saved = attached_presentation_saved
        .data
        .get(0)
        .ok_or("Presentation data is empty")?
        .json
        .verifiable_credential
        .as_ref()
        .ok_or("Credentials not attached")?
        .get(0)
        .ok_or("Credentials are empty")?
        .r#type
        .get(0)
        .ok_or("Credential type not found")?;

    assert_eq!(data, data_saved);

    return Ok(());
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
                            name: "US Passport".to_string(),
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
                                    id: "Some_field_id".to_string(),
                                    purpose: None,
                                    predicate: None,
                                    filter: [
                                        ("type".to_string(), "date".to_string()),
                                        ("minimum".to_string(), "1999-5-16".to_string()),
                                    ]
                                    .iter()
                                    .cloned()
                                    .collect(),
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

    let received: VadeDidCommPluginOutput<MessageWithBody<PresentationExchangeData>> =
        serde_json::from_str(result)?;

    let received_proposal = received
        .message
        .body
        .ok_or("send DIDComm request does not return presentation request".to_owned())?;

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

    let proposal_data_saved = get_presentation(sender, receiver, id, state)?.proposal_attach;
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
