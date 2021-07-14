use crate::{get_step_output, receive_step, send_step, MessageWithBody, Protocol, StepResult};
use serde::{Deserialize, Serialize};

/// Struct for parsing incoming ping messages.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PingBody {
    response_requested: Option<bool>,
}

/// Creates a new ping_pong protocol and maps the specific step handler functions.
///
/// # Returns
/// * `Protocol` - the new ping pong protocol handler
pub fn get_ping_pong_protocol() -> Protocol {
    let mut protocol = Protocol {
        name: String::from("trust_ping"),
        steps: Vec::new(),
    };

    protocol.steps.push(send_step("ping", send_ping));
    protocol.steps.push(send_step("ping_response", send_pong));
    protocol.steps.push(receive_step("ping", receive_ping));
    protocol
        .steps
        .push(receive_step("ping_response", receive_pong));

    return protocol;
}

/// Protocol handler for direction: `send`, type: `trust_ping/ping`
pub fn send_ping(message: &str) -> StepResult {
    let mut parsed_message: MessageWithBody<PingBody> = serde_json::from_str(message)?;
    parsed_message.body = Some(PingBody {
        response_requested: Some(true),
    });
    return get_step_output(&serde_json::to_string(&parsed_message)?, "{}");
}

/// Protocol handler for direction: `send`, type: `trust_ping/pong`
pub fn send_pong(message: &str) -> StepResult {
    return get_step_output(message, "{}");
}

/// Protocol handler for direction: `receive`, type: `trust_ping/ping`
pub fn receive_ping(message: &str) -> StepResult {
    return get_step_output(message, "{}");
}

/// Protocol handler for direction: `receive`, type: `trust_ping/pong`
pub fn receive_pong(message: &str) -> StepResult {
    return get_step_output(message, "{}");
}
