use crate::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody, PresentationData},
    get_from_to_from_message,
    presentation::{get_presentation, save_presentation},
    protocols::protocol::{generate_step_output, StepResult},
};

use super::helper::{
    get_present_proof_info_from_message, get_present_proof_message, PresentProofType,
};

/// protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/request-presentation`
/// Uses the protocols/present_proof/helper.rs/get_present_proof_message to construct the request message,
pub fn send_request_presentation(message: &str) -> StepResult {
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

    let request_message = get_present_proof_message(
        PresentProofType::RequestPresentation,
        &exchange_info.from,
        &exchange_info.to,
        presentation_data.clone(),
    )?;

    save_presentation(
        &exchange_info.from,
        &exchange_info.to,
        &serde_json::to_string(&presentation_data)?,
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

/// protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/presentation`
/// Receives the presentation from prover and updates in db
pub fn receive_presentation(message: &str) -> StepResult {
    println!("message {}", message);
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
    let exchange_info = get_present_proof_info_from_message(parsed_message)?;
    let base_info = get_from_to_from_message(base_message)?;

    let presentation_data = exchange_info
        .presentation_data
        .ok_or("Presentation data not provided.")?;
    let saved_presentation_data = get_presentation(&base_info.from, &base_info.to)?;
    save_presentation(
        &base_info.from,
        &base_info.to,
        &serde_json::to_string(&presentation_data)?,
    )?;
    let presentation = saved_presentation_data
        .presentation_attach
        .ok_or("Presentation request not attached.")?;
    let metadata = presentation.get(0).ok_or("Request data not attached")?;
    generate_step_output(message, &serde_json::to_string(&metadata)?)
}

/// protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/propose-presentation`
/// Receives the proposal for new presentation request from prover
pub fn receive_propose_presentation(message: &str) -> StepResult {
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
    let exchange_info = get_present_proof_info_from_message(parsed_message)?;
    let base_info = get_from_to_from_message(base_message)?;

    let presentation_data = exchange_info
        .presentation_data
        .ok_or("Presentation data not provided.")?;

    save_presentation(
        &base_info.from,
        &base_info.to,
        &serde_json::to_string(&presentation_data)?,
    )?;
    let saved_presentation_data = get_presentation(&base_info.from, &base_info.to)?;
    let presentation_proposal = saved_presentation_data
        .presentation_proposal
        .ok_or("Presentation request not attached.")?;
    generate_step_output(message, &serde_json::to_string(&presentation_proposal)?)
}
