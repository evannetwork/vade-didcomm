use crate::{Direction, Protocol, get_ping_pong_protocol, message::{Message}, protocol::get_did_exchange_protocol, utils::SyncResult};

pub struct ProtocolHandleOutput {
    pub direction: Direction,
    pub encrypt: bool,
    pub protocol: String,
    pub metadata: String,
    pub step: String,
}

pub struct ReceiveResult {
    pub protocol: String,
}

fn handle_protocol(
    message: &mut Message,
    direction: Direction,
) -> SyncResult<ProtocolHandleOutput> {
    let m_type = message
        .r#type
        .as_ref()
        .ok_or("message type is missing".to_string())?
        .to_owned();
    let mut protocol_name: String = String::from("unknown");
    let mut stepName: String = String::from("unknown");
    let mut encrypt = true;
    let mut metadata: String = String::from("{}");
    let protocols: [&Protocol; 2] = [
        &get_did_exchange_protocol(),
        &get_ping_pong_protocol(),
    ];

    for i in 0..protocols.len() {
        let protocol = &protocols[i];
        println!("{}", &protocol.name);
        if m_type.contains(&protocol.name) {
            protocol_name = String::from(&protocol.name);

            for x in 0..protocol.steps.len() {
                let step = &protocol.steps[x];

                if step.direction == direction && m_type.contains(&step.name) {
                    stepName = String::from(&step.name);
                    let step_outcome = (step.handler)(message)?;
                    encrypt = step_outcome.encrypt;
                    metadata = step_outcome.metadata;
                    break;
                }
            }
        }

        if protocol_name != "unknown" && stepName != "unknown" {
            break;
        }
    }

    return Ok(ProtocolHandleOutput {
        direction,
        encrypt: encrypt,
        protocol: protocol_name,
        metadata,
        step: stepName.to_owned(),
    });
}

pub struct ProtocolHandler {}

impl ProtocolHandler {
    pub fn before_send(
        message: &mut Message,
    ) -> SyncResult<ProtocolHandleOutput> {
        return handle_protocol(message, Direction::SEND);
    }

    pub fn after_receive(
        message: &mut Message,
    ) -> SyncResult<ProtocolHandleOutput> {
        return handle_protocol(message, Direction::RECEIVE);
    }
}
