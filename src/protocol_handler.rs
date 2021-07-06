use crate::{Direction, Protocol, get_ping_pong_protocol, message::{Message}, utils::SyncResult};

pub struct ProtocolHandleResult {
    pub protocol: String,
    pub step: String,
    pub encrypt: bool,
    pub direction: Direction,
}

pub struct ReceiveResult {
    pub protocol: String,
}

fn handle_protocol(
    message: &mut Message,
    direction: Direction,
) -> SyncResult<ProtocolHandleResult> {
    let m_type = message
        .r#type
        .as_ref()
        .ok_or("message type is missing".to_string())?
        .to_owned();
    let mut protocol_name: String = String::from("unknown");
    let mut stepName: String = String::from("unknown");
    let encrypt = &mut true;
    let protocols: [&Protocol; 1] = [
        &get_ping_pong_protocol(),
    ];

    for i in 0..protocols.len() {
        let protocol = &protocols[i];
        println!("{}", &protocol.name);
        if m_type.contains(&protocol.name) {
            protocol_name = String::from(&protocol.name);

            for x in 0..protocol.steps.len() {
                let step = &protocol.steps[x];

                if matches!(&step.direction, direction) && m_type.contains(&step.name) {
                    stepName = String::from(&step.name);
                    (step.handler)(message, encrypt);
                    break;
                }
            }
        }

        if protocol_name != "unknown" && stepName != "unknown" {
            break;
        }
    }

    println!("-------------testest: {}", encrypt);

    return Ok(ProtocolHandleResult {
        direction,
        encrypt: encrypt.to_owned(),
        protocol: protocol_name,
        step: stepName.to_owned(),
    });
}

pub struct ProtocolHandler {}

impl ProtocolHandler {
    pub fn before_send(
        message: &mut Message,
    ) -> SyncResult<ProtocolHandleResult> {
        return handle_protocol(message, Direction::SEND);
    }

    pub fn after_receive(
        message: &mut Message,
    ) -> SyncResult<ProtocolHandleResult> {
        return handle_protocol(message, Direction::RECEIVE);
    }
}
