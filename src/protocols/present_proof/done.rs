use crate::{
    datatypes::{BaseMessage, PRESENT_PROOF_PROTOCOL_URL},
    protocols::protocol::{generate_step_output, StepResult},
};

/// protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn send_presentation_ack(message: &str) -> StepResult {
    let mut parsed_message: BaseMessage = serde_json::from_str(message)?;
    parsed_message.r#type = format!("{}/ack", PRESENT_PROOF_PROTOCOL_URL);

    return generate_step_output(&serde_json::to_string(&parsed_message)?, "{}");
}

/// protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn receive_presentation_ack(message: &str) -> StepResult {
    return generate_step_output(message, "{}");
}
