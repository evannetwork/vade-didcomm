use crate::{ Message, Protocol, receive_step, send_step};

macro_rules! sf {
    ( $var:expr ) => ( String::from($var) );
}

pub fn send_ping(message: &mut Message, _encrypt: &mut bool) {
    message.body = format!(
        r#"{{
            "response_requested": true
        }}"#,
    );
}

pub fn send_pong(message: &mut Message, _encrypt: &mut bool) {
    let thread_id = message.other.get("thread_id");
    thread_id.ok_or("PING-PONG Message does not contain header thread_id");
}

pub fn receive_ping(message: &mut Message, _encrypt: &mut bool) {
    let thread_id = message.other.get("thread_id");
    thread_id.ok_or("PING-PONG Message does not contain header thread_id");
}

pub fn receive_pong(message: &mut Message, _encrypt: &mut bool) {
    let thread_id = message.other.get("thread_id");
    thread_id.ok_or("PING-PONG Message does not contain header thread_id");
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
