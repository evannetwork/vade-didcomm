use crate::{
    datatypes::{MessageDirection, MessageWithType, ProtocolHandleOutput},
    protocols::{
        did_exchange::generate_did_exchange_protocol, pingpong::generate_ping_pong_protocol,
        protocol::Protocol,
    },
    utils::SyncResult,
};

pub struct ProtocolHandler {}
impl ProtocolHandler {
    /// Runs all protocol handlers for a message, to prepare it for sending. Each protocol can enrich
    /// the message with step specific information or can store things like communication keys.
    ///
    /// # Arguments
    /// * `message` - message string (should match message.rs/ExtendedMessage)
    ///
    /// # Returns
    /// * `ProtocolHandleOutput` - general information about the analyzed protocol step
    pub fn before_send(message: &str) -> SyncResult<ProtocolHandleOutput> {
        return handle_protocol(message, MessageDirection::SEND);
    }

    /// Runs all protocol handlers for a message, to analyze it after receiving and decryption.
    /// Each protocol can enrich the message with step specific information or can store things
    /// like communication keys.
    ///
    /// # Arguments
    /// * `message` - message string (should match message.rs/ExtendedMessage))
    ///
    /// # Returns
    /// * `ProtocolHandleOutput` - general information about the analyzed protocol step
    pub fn after_receive(message: &str) -> SyncResult<ProtocolHandleOutput> {
        return handle_protocol(message, MessageDirection::RECEIVE);
    }
}

/// General protocol step handler for analyzing messages with a direction (incoming / outgoing).
/// It analyse the message type and checks if a step with a specific direction is configured.
/// When a step is found, the logic will be executed and no other handler will be searched.
fn handle_protocol(message: &str, direction: MessageDirection) -> SyncResult<ProtocolHandleOutput> {
    let parsed_message: MessageWithType = serde_json::from_str(message)?;
    let m_type = parsed_message.r#type.to_owned();
    // handle multiple protocols dynamically
    let protocols: [&Protocol; 2] = [
        &generate_did_exchange_protocol(),
        &generate_ping_pong_protocol(),
    ];
    // protocol results
    let mut protocol_name: String = String::from("unknown");
    let mut step_name: String = String::from("unknown");
    let mut encrypt = true;
    let mut metadata: String = String::from("{}");
    let mut message_output: String = String::from(message);

    for i in 0..protocols.len() {
        let protocol = &protocols[i];

        // check if the type includes the protocol name
        if m_type.contains(&protocol.name) {
            protocol_name = String::from(&protocol.name);

            for x in 0..protocol.steps.len() {
                let step = &protocol.steps[x];

                // check for configured step names and directions
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
