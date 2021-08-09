use crate::{
    datatypes::{PresentProofReq, MessageWithBody},
    get_from_to_from_message,
    presentation::{get_presentation, save_presentation},
    protocols::protocol::{generate_step_output, StepResult},
};

use super::helper::{get_present_proof_message, get_present_proof_info_from_message, PresentProofType};

/// protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/request-presentation`
/// Uses the protocols/present_proof/helper.rs/get_present_proof_message to construct the message,
/// that should be sent.
pub fn send_presentation(message: &str) -> StepResult {
    let parsed_message: PresentProofReq = serde_json::from_str(message)?;
    let exchange_info = get_from_to_from_message(parsed_message.baseMessage)?; 
    let saved_presentation_data = get_presentation(&exchange_info.from,&exchange_info.to,);
    let request_message = get_present_proof_message(
        PresentProofType::PRESENTATION,
        &exchange_info.from,
        &exchange_info.to,
        &parsed_message.requestPresentation,
    )?;
 
    return generate_step_output(&serde_json::to_string(&request_message)?, &saved_presentation_data.ok().unwrap_or_else(|| String::from("{}")));
}

/// protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/presentation`
/// Receives the presentation and updates the existing presentation for this DID in
/// the db.
pub fn receive_request_presentation(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<String> = serde_json::from_str(message)?;
    let exchange_info = get_present_proof_info_from_message(parsed_message)?;
    save_presentation(
        &exchange_info.to,
        &exchange_info.from,
        &exchange_info.presentation_data,
    )?;
    return generate_step_output(message,  &exchange_info.presentation_data);
}


/// protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/propose-presentation`
/// Uses the protocols/present_proof/helper.rs/get_present_proof_message to construct the request message,
/// that should be sent.
pub fn send_propose_presentation(message: &str) -> StepResult {
    let parsed_message: PresentProofReq = serde_json::from_str(message)?;
    let exchange_info = get_from_to_from_message(parsed_message.baseMessage)?; 
    let saved_presentation_data = get_presentation(&exchange_info.from,&exchange_info.to,);
    let request_message = get_present_proof_message(
        PresentProofType::PROPOSE_PRESENTATION,
        &exchange_info.from,
        &exchange_info.to,
        &parsed_message.requestPresentation,
    )?;
 
    return generate_step_output(&serde_json::to_string(&request_message)?, &saved_presentation_data.ok().unwrap_or_else(|| String::from("{}")));
}