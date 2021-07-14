use crate::{get_step_output, protocol::DID_EXCHANGE_PROTOCOL_URL, BaseMessage, StepResult};

pub fn send_complete(message: &str) -> StepResult {
    let mut parsed_message: BaseMessage = serde_json::from_str(message)?;
    parsed_message.r#type = format!("{}/complete", DID_EXCHANGE_PROTOCOL_URL);

    return get_step_output(&serde_json::to_string(&parsed_message)?, "{}");
}

pub fn receive_complete(message: &str) -> StepResult {
    return get_step_output(message, "{}");
}
