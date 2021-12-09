use std::collections::HashMap;

use super::helper::{
    get_present_proof_info_from_message,
    get_present_proof_message,
    PresentProofType,
};
use crate::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody},
    get_from_to_from_message,
    protocols::{
        present_proof::{
            datatypes::{PresentationData, State, UserType},
            presentation::{get_current_state, save_presentation, save_state},
        },
        protocol::{generate_step_output, StepResult},
    },
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/request-presentation`
pub fn send_request_presentation(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        body: HashMap::new(),
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(&base_message)?;

    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation data not provided.")?,
    )?;
    let presentation_data: PresentationData = serde_json::from_str(data)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let request_message = get_present_proof_message(
        PresentProofType::RequestPresentation,
        &exchange_info.from,
        &exchange_info.to,
        presentation_data.clone(),
        &thid,
    )?;

    let current_state: State = get_current_state(&thid, &UserType::Verifier)?.parse()?;
    match current_state {
        State::PresentationProposalReceived | State::Unknown => {
            save_state(&thid, &State::PresentationRequested, &UserType::Verifier)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::PresentationRequested
            )))
        }
    };

    save_presentation(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&presentation_data)?,
        &State::PresentationRequested,
    )?;

    let presentation_request = presentation_data
        .presentation_attach
        .ok_or("Presentation request not attached.")?;
    let metadata = presentation_request
        .get(0)
        .ok_or("Request data not attached")?;
    generate_step_output(
        &serde_json::to_string(&request_message)?,
        &serde_json::to_string(metadata)?,
    )
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/presentation`
pub fn receive_presentation(_options: &str, message: &str) -> StepResult {
    let parsed_message: MessageWithBody<PresentationData> = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        body: HashMap::new(),
        from: parsed_message.from.clone(),
        r#type: parsed_message.r#type.clone(),
        to: Some(
            parsed_message
                .to
                .clone()
                .ok_or("To DID not provided")?
                .to_vec(),
        ),
    };
    let thid = parsed_message
        .thid
        .to_owned()
        .ok_or("Thread id can't be empty")?;
    let base_info = get_from_to_from_message(&base_message)?;

    let current_state: State = get_current_state(&thid, &UserType::Verifier)?.parse()?;

    match current_state {
        State::PresentationRequested => {
            save_state(&thid, &State::PresentationReceived, &UserType::Verifier)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::PresentationReceived
            )))
        }
    };

    let exchange_info = get_present_proof_info_from_message(parsed_message)?;

    let presentation_data = exchange_info
        .presentation_data
        .ok_or("Presentation data not provided.")?;

    save_presentation(
        &base_info.from,
        &base_info.to,
        &thid,
        &serde_json::to_string(&presentation_data)?,
        &State::PresentationReceived,
    )?;
    let presentation = presentation_data
        .presentation_attach
        .ok_or("Presentation request not attached.")?;
    let metadata = presentation.get(0).ok_or("Request data not attached")?;
    generate_step_output(message, &serde_json::to_string(&metadata)?)
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/propose-presentation`
pub fn receive_propose_presentation(_options: &str, message: &str) -> StepResult {
    let parsed_message: MessageWithBody<PresentationData> = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        body: HashMap::new(),
        from: parsed_message.from.clone(),
        r#type: parsed_message.r#type.clone(),
        to: Some(
            parsed_message
                .to
                .clone()
                .ok_or("To DID not provided")?
                .to_vec(),
        ),
    };

    let base_info = get_from_to_from_message(&base_message)?;
    let thid = parsed_message
        .thid
        .to_owned()
        .ok_or("Thread id can't be empty")?;

    let exchange_info = get_present_proof_info_from_message(parsed_message)?;

    let presentation_data = exchange_info
        .presentation_data
        .ok_or("Presentation data not provided.")?;

    let current_state: State = get_current_state(&thid, &UserType::Verifier)?.parse()?;
    match current_state {
        State::PresentationRequested | State::Unknown => save_state(
            &thid,
            &State::PresentationProposalReceived,
            &UserType::Verifier,
        )?,
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::PresentationProposalReceived
            )))
        }
    };

    save_presentation(
        &base_info.from,
        &base_info.to,
        &thid,
        &serde_json::to_string(&presentation_data)?,
        &State::PresentationProposalReceived,
    )?;

    let presentation_proposal = presentation_data
        .presentation_proposal
        .ok_or("Presentation request not attached.")?;
    let attribute = presentation_proposal
        .attribute
        .ok_or("No Attributes provided")?;
    let metadata = attribute
        .get(0)
        .ok_or("Attribute data should be provided")?;
    generate_step_output(message, &serde_json::to_string(metadata)?)
}
