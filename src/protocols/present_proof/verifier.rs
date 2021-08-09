use crate::{
    datatypes::{PresentProofReq, MessageWithBody},
    get_from_to_from_message,
    presentation::{save_presentation,get_presentation},
    protocols::protocol::{generate_step_output, StepResult},
};

use super::helper::{get_present_proof_message, get_present_proof_info_from_message, PresentProofType};

/// protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/request-presentation`
/// Uses the protocols/present_proof/helper.rs/get_present_proof_message to construct the request message,
pub fn send_request_presentation(message: &str) -> StepResult {
    let parsed_message: PresentProofReq = serde_json::from_str(message)?;
    let exchange_info = get_from_to_from_message(parsed_message.baseMessage)?; 

    let request_message = get_present_proof_message(
        PresentProofType::REQUEST_PRESENTATION,
        &exchange_info.from,
        &exchange_info.to,
        &parsed_message.requestPresentation,
    )?;
 
    return generate_step_output(&serde_json::to_string(&request_message)?, &parsed_message.requestPresentation);
}

/// protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/presentation`
/// Receives the presentation from prover and updates in db
pub fn receive_presentation(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<String> = serde_json::from_str(message)?;
    let exchange_info = get_present_proof_info_from_message(parsed_message)?;
    let saved_presentation_data = get_presentation(&exchange_info.from,&exchange_info.to,);
    save_presentation(
        &exchange_info.to,
        &exchange_info.from,
        &exchange_info.presentation_data,
    )?;
    return generate_step_output(message, &saved_presentation_data.ok().unwrap_or_else(|| String::from("{}")));
}


/// protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/propose-presentation`
/// Receives the proposal for new presentation request from prover
pub fn receive_propose_presentation(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<String> = serde_json::from_str(message)?;
    let exchange_info = get_present_proof_info_from_message(parsed_message)?;
    let saved_presentation_data = get_presentation(&exchange_info.from,&exchange_info.to,);
    save_presentation(
        &exchange_info.to,
        &exchange_info.from,
        &exchange_info.presentation_data,
    )?;
    return generate_step_output(message, &saved_presentation_data.ok().unwrap_or_else(|| String::from("{}")));
}