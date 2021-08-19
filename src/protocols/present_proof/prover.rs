use crate::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody},
    get_from_to_from_message,
    protocols::present_proof::datatypes::{PresentationData, State, UserType},
    protocols::present_proof::presentation::{get_current_state, save_presentation, save_state},
    protocols::protocol::{generate_step_output, StepResult},
};

use super::helper::{
    get_present_proof_info_from_message, get_present_proof_message, PresentProofType,
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/presentation`
pub fn send_presentation(message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(base_message)?;

    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation data not provided.")?,
    )?;
    let presentation_data: PresentationData = serde_json::from_str(&data)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &UserType::Prover)?.parse()?;

    let result = match current_state {
        State::PresentationRequestReceived => {
            save_state(&thid, &State::PresentationSent, &UserType::Prover)
        }
        _ => Err(Box::from(format!(
            "State from {} to {} not allowed",
            current_state,
            State::PresentationSent
        ))),
    };

    result.map_err(|err| format!("Error while processing step: {:?}", err))?;

    let request_message = get_present_proof_message(
        PresentProofType::Presentation,
        &exchange_info.from,
        &exchange_info.to,
        presentation_data.clone(),
        &thid,
    )?;

    save_presentation(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&presentation_data)?,
        &State::PresentationSent,
    )?;

    let presentation_request = presentation_data
        .presentation_attach
        .ok_or("Presentation data not attached.")?;
    let metadata = presentation_request
        .get(0)
        .ok_or("Request data not attached")?;

    generate_step_output(
        &serde_json::to_string(&request_message)?,
        &serde_json::to_string(metadata)?,
    )
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/request_presentation`
pub fn receive_request_presentation(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<PresentationData> = serde_json::from_str(message)?;

    let base_message: BaseMessage = BaseMessage {
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

    let exchange_info = get_present_proof_info_from_message(parsed_message)?;
    let base_info = get_from_to_from_message(base_message)?;
    let presentation_data = exchange_info
        .presentation_data
        .ok_or("Presentation data not provided.")?;

    let current_state: State = get_current_state(&thid, &UserType::Prover)?.parse()?;

    let result = match current_state {
        State::PresentationProposed | State::Unknown => save_state(
            &thid,
            &State::PresentationRequestReceived,
            &UserType::Prover,
        ),
        _ => Err(Box::from(format!(
            "State from {} to {} not allowed",
            current_state,
            State::PresentationRequestReceived
        ))),
    };

    result.map_err(|err| format!("Error while processing step: {:?}", err))?;

    save_presentation(
        &base_info.to,
        &base_info.from,
        &thid,
        &serde_json::to_string(&presentation_data)?,
        &State::PresentationRequestReceived,
    )?;

    let presentation_request = presentation_data
        .presentation_attach
        .ok_or("Presentation request not attached.")?;
    let metadata = presentation_request
        .get(0)
        .ok_or("Request data not attached")?;
    generate_step_output(message, &serde_json::to_string(metadata)?)
}

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/propose-presentation`
pub fn send_propose_presentation(message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(base_message)?;

    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation data not provided.")?,
    )?;
    let presentation_data: PresentationData = serde_json::from_str(&data)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &UserType::Prover)?.parse()?;

    let result = match current_state {
        State::PresentationRequestReceived | State::Unknown => {
            save_state(&thid, &State::PresentationProposed, &UserType::Prover)
        }
        _ => Err(Box::from(format!(
            "State from {} to {} not allowed",
            current_state,
            State::PresentationProposed
        ))),
    };

    result.map_err(|err| format!("Error while processing step: {:?}", err))?;

    let request_message = get_present_proof_message(
        PresentProofType::ProposePresentation,
        &exchange_info.from,
        &exchange_info.to,
        presentation_data.clone(),
        &thid,
    )?;

    save_presentation(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&presentation_data)?,
        &State::PresentationProposed,
    )?;

    let presentation_proposal = presentation_data
        .presentation_proposal
        .ok_or("Presentation data not attached.")?;

    let attribute = presentation_proposal
        .attribute
        .ok_or("No Attributes provided")?;
    let metadata = attribute
        .get(0)
        .ok_or("Attribute data should be provided")?;
    generate_step_output(
        &serde_json::to_string(&request_message)?,
        &serde_json::to_string(metadata)?,
    )
}
