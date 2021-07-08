use crate::{Message, Protocol, StepOutput, StepResult, receive_step, send_step};

macro_rules! sf {
    ( $var:expr ) => ( String::from($var) );
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

pub fn send_ping(message: &mut Message) -> StepResult {
    message.body = format!(
        r#"{{
            "response_requested": true
        }}"#,
    );
    return Ok(StepOutput { encrypt: true, metadata: String::from("{}") });
}

pub fn send_pong(message: &mut Message) -> StepResult {
    let thread_id = message.other.get("thread_id");
    thread_id.ok_or("PING-PONG Message does not contain header thread_id");
    return Ok(StepOutput { encrypt: true, metadata: String::from("{}") });
}

pub fn receive_ping(message: &mut Message) -> StepResult {
    let thread_id = message.other.get("thread_id");
    thread_id.ok_or("PING-PONG Message does not contain header thread_id");
    return Ok(StepOutput { encrypt: true, metadata: String::from("{}") });
}

pub fn receive_pong(message: &mut Message) -> StepResult {
    let thread_id = message.other.get("thread_id");
    thread_id.ok_or("PING-PONG Message does not contain header thread_id");
    return Ok(StepOutput { encrypt: true, metadata: String::from("{}") });
}
