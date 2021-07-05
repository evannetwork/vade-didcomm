use crate::{Protocol, message::{Message}, PING_PONG_PROTOCOL};

pub struct ProtocolHandleResult {
    pub protocol: String,
    pub step: String,
    pub encrypt: bool,
    pub direction: String,
}

pub struct ReceiveResult {
    pub protocol: String,
}

static PROTOCOLS: [&Protocol; 1] = [
    &PING_PONG_PROTOCOL,
];

fn handle_protocol(
    message: &mut Message,
    direction: String,
) -> Result<ProtocolHandleResult, Box<dyn std::error::Error>> {
    let m_type = message
        .r#type
        .as_ref()
        .ok_or("message type is missing".to_string())?
        .to_owned();
    let mut protocolName: String = String::from("unknown");
    let mut step: String = String::from("unknown");
    let encrypt = &mut true;

    for i in 0..PROTOCOLS.len() {
        if m_type.contains(&PROTOCOLS[i].name) {
            let protocol = &PROTOCOLS[i];
            protocolName = String::from(&protocol.name);

            for x in 0..protocol.steps.len() {
                let protocolStep = &protocol.steps[x];

                if m_type.contains(&protocolStep.name) {
                    step = String::from(&protocolStep.name);
                    (protocolStep.handler)(message, encrypt);
                    break;
                }
            }
        }

        if protocolName != "unknown" && step != "unknown" {
            break;
        }
    }

    return Ok(ProtocolHandleResult {
        direction,
        encrypt: encrypt.to_owned(),
        protocol: protocolName,
        step,
    });
}

pub struct ProtocolHandler {}

impl ProtocolHandler {
    pub fn before_send(
        message: &mut Message,
    ) -> Result<ProtocolHandleResult, Box<dyn std::error::Error>> {
        return handle_protocol(message, String::from("send"));
    }

    pub fn after_receive(
        message: &mut Message,
    ) -> Result<ProtocolHandleResult, Box<dyn std::error::Error>> {
        return handle_protocol(message, String::from("receive"));
    }
}
