use crate::{Message, Protocol, ProtocolConfig};

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

    protocol.steps.push(ProtocolConfig { name: sf!("send_ping"), handler: send_ping });
    protocol.steps.push(ProtocolConfig { name: sf!("send_ping_response"), handler: send_pong });
    protocol.steps.push(ProtocolConfig { name: sf!("receive_ping"), handler: receive_ping });
    protocol.steps.push(ProtocolConfig { name: sf!("receive_ping_response"), handler: receive_pong });

    return protocol;
}

pub static PING_PONG_PROTOCOL: Protocol = get_ping_pong_protocol();
