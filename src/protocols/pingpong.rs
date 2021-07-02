use crate::Message;

pub fn send_ping(message: &mut Message) {
    message.body = format!(
        r#"{{
            "response_requested": true
        }}"#,
    );
}

pub fn send_receive(message: &mut Message) {
    let thread_id = message.other.get("thread_id");
    thread_id.ok_or("PING-PONG Message does not contain header thread_id");
}
