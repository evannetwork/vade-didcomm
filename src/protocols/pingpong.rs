use serde::{Deserialize, Serialize};
use crate::{MessageWithBody, Protocol, StepResult, get_step_output, receive_step, send_step};

macro_rules! sf {
    ( $var:expr ) => ( String::from($var) );
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PingBody {
    response_requested: Option<bool>,
}

pub fn get_ping_pong_protocol() -> Protocol {
    let mut protocol = Protocol {
        name: sf!("trust_ping"),
        steps: Vec::new(),
    };

    protocol.steps.push(send_step("ping", send_ping));
    protocol.steps.push(send_step("ping_response", send_pong));
    protocol.steps.push(receive_step("ping", receive_ping));
    protocol.steps.push(receive_step("ping_response", receive_pong));

    return protocol;
}

pub fn send_ping(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<PingBody> = serde_json::from_str(message)?;
    parsed_message.body.response_requested = Some(true);
    return get_step_output("{}", &serde_json::to_string(&parsed_message)?);
}

pub fn send_pong(message: &str) -> StepResult {
    return get_step_output("{}", message);
}

pub fn receive_ping(message: &str) -> StepResult {
    return get_step_output("{}", message);
}

pub fn receive_pong(message: &str) -> StepResult {
    return get_step_output("{}", message);
}
