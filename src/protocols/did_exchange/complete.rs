use crate::{
    get_step_output, protocols::did_exchange::did_exchange::DID_EXCHANGE_PROTOCOL_URL, BaseMessage,
    StepResult,
};

/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/complete`
/// just ensures to set the correct message type, before the message will be sent (first time for
/// did exchange, that a encrypted message will be sent)
pub fn send_complete(message: &str) -> StepResult {
    let mut parsed_message: BaseMessage = serde_json::from_str(message)?;
    parsed_message.r#type = format!("{}/complete", DID_EXCHANGE_PROTOCOL_URL);

    return get_step_output(&serde_json::to_string(&parsed_message)?, "{}");
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/complete`
pub fn receive_complete(message: &str) -> StepResult {
    return get_step_output(message, "{}");
}
