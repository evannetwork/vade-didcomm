use crate::{
    datatypes::{Ack, ExtendedMessage},
    protocols::protocol::{generate_step_output, StepResult},
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn send_presentation_ack(message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation data not provided.")?,
    )?;
    let ack: Ack = serde_json::from_str(&data)?;

    generate_step_output(&serde_json::to_string(&ack)?, "{}")
}
/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn receive_presentation_ack(message: &str) -> StepResult {
    generate_step_output(message, "{}")
}