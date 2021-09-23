use crate::{
    datatypes::{MessageDirection, MessageWithType, ProtocolHandleOutput},
    protocols::{
        did_exchange::generate_did_exchange_protocol,
        issue_credential::generate_issue_credential_protocol,
        pingpong::generate_ping_pong_protocol, present_proof::generate_present_proof_protocol,
        presentation_exchange::generate_presentation_exchange_protocol, protocol::Protocol,
    },
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
    pub fn before_send(message: &str) -> Result<ProtocolHandleOutput, Box<dyn std::error::Error>> {
        handle_protocol(message, MessageDirection::Send)
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
    pub fn after_receive(
        message: &str,
    ) -> Result<ProtocolHandleOutput, Box<dyn std::error::Error>> {
        handle_protocol(message, MessageDirection::Receive)
    }
}

/// General protocol step handler for analyzing messages with a direction (incoming / outgoing).
/// It analyse the message type and checks if a step with a specific direction is configured.
/// When a step is found, the logic will be executed and no other handler will be searched.
fn handle_protocol(
    message: &str,
    direction: MessageDirection,
) -> Result<ProtocolHandleOutput, Box<dyn std::error::Error>> {
    let parsed_message: MessageWithType = serde_json::from_str(message)?;
    let m_type = parsed_message.r#type;
    // handle multiple protocols dynamically
    let protocols: [&Protocol; 5] = [
        &generate_did_exchange_protocol(),
        &generate_ping_pong_protocol(),
        &generate_present_proof_protocol(),
        &generate_issue_credential_protocol(),
        &generate_presentation_exchange_protocol(),
    ];
    // protocol results
    let mut protocol_name: String = String::from("unknown");
    let mut step_name: String = String::from("unknown");
    let mut encrypt = true;
    let mut metadata: String = String::from("{}");
    let mut message_output: String = String::from(message);

    for protocol in &protocols {
        // check if the type includes the protocol name
        if m_type.contains(&protocol.name) {
            protocol_name = String::from(&protocol.name);

            for x in 0..protocol.steps.len() {
                let step = &protocol.steps[x];
                let protocol_type = format!("{}/{}", protocol_name, step.name);
                // check for configured step names and directions
                if step.direction == direction && m_type.contains(&protocol_type) {
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

    Ok(ProtocolHandleOutput {
        direction,
        encrypt,
        protocol: protocol_name,
        metadata,
        message: message_output,
        step: step_name,
    })
}
