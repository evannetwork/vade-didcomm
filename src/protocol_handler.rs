use crate::{Direction, MessageWithType, Protocol, get_ping_pong_protocol, protocol::get_did_exchange_protocol, utils::SyncResult};

/// Output of a protocol step. Specifies, if a message should be encrypted. Metadata is generic stringified
/// json, that contains protocol step specific information.
pub struct ProtocolHandleOutput {
    pub direction: Direction,
    pub encrypt: bool,
    pub protocol: String,
    pub metadata: String,
    pub message: String,
    pub step: String,
}

pub struct ReceiveResult {
    pub protocol: String,
}

pub struct ProtocolHandler {}
impl ProtocolHandler {
    /// Runs all protocol handlers for a message, that should be sent.
    ///
    /// # Arguments
    /// * `message` - message string (should match message.rs/)
    ///
    /// # Returns
    /// * `ProtocolHandleOutput` - general information about the analyzed protocol step
    pub fn before_send(
        message: &str,
    ) -> SyncResult<ProtocolHandleOutput> {
        return handle_protocol(message, Direction::SEND);
    }

    pub fn after_receive(
        message: &str,
    ) -> SyncResult<ProtocolHandleOutput> {
        return handle_protocol(message, Direction::RECEIVE);
    }
}

fn handle_protocol(
    message: &str,
    direction: Direction,
) -> SyncResult<ProtocolHandleOutput> {
    let parsed_message: MessageWithType = serde_json::from_str(message)?;
    let m_type = parsed_message.r#type.to_owned();
    // handle multiple protocols dynamically
    let protocols: [&Protocol; 2] = [
        &get_did_exchange_protocol(),
        &get_ping_pong_protocol(),
    ];
    // protocol results
    let mut protocol_name: String = String::from("unknown");
    let mut step_name: String = String::from("unknown");
    let mut encrypt = true;
    let mut metadata: String = String::from("{}");
    let mut message_output: String = String::from(message);

    for i in 0..protocols.len() {
        let protocol = &protocols[i];
        if m_type.contains(&protocol.name) {
            protocol_name = String::from(&protocol.name);

            for x in 0..protocol.steps.len() {
                let step = &protocol.steps[x];

                if step.direction == direction && m_type.contains(&step.name) {
                    let step_outcome = (step.handler)(message)?;
                    encrypt = step_outcome.encrypt;
                    metadata = step_outcome.metadata;
                    message_output = step_outcome.message;
                    step_name = String::from(&step.name);
                    break;
                }
            }
        }

        if protocol_name != "unknown" && step_name != "unknown" {
            break;
        }
    }

    return Ok(ProtocolHandleOutput {
        direction,
        encrypt,
        protocol: protocol_name,
        metadata,
        message: message_output,
        step: step_name.to_owned(),
    });
}
