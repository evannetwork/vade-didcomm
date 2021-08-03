use crate::datatypes::MessageDirection;

/// Each protocol are constructed by a name and multiple steps. The protocol handler will iterate over
/// all registered protocols and checks, if the name exists in the DIDComm message type. Afterwards
/// each step will be checked against the type as well.
///
/// Example:
///     - name          -> https://didcomm.org/didexchange/1.0
///     - step[0].name  -> request
///     - message 1 -> type = trust_ping/ping -> protocol step will not be executed
///     - message 2 -> type = https://didcomm.org/didexchange/1.0/request -> protocol step will be executed
pub struct Protocol {
    pub name: String,
    pub steps: Vec<ProtocolStep>,
}

/// Each protocol step specifies the direction and the name, when the handler function will be executed.
/// The step handler can take the incoming message, parse it and can return an adjusted message to be
/// returned to the user.
pub struct ProtocolStep {
    pub direction: MessageDirection,
    pub handler: fn(message: &str) -> StepResult,
    pub name: String,
}

/// Result of each protocol step. Includes the custom stringified metadata, the modified message and
/// a bool flag, if the message should be encrypted (ignored for direction == receive).
pub struct StepOutput {
    pub encrypt: bool,
    pub metadata: String,
    pub message: String,
}

pub type StepResult = Result<StepOutput, Box<dyn std::error::Error>>;

/// Shorthand generator for a protocol step, with direction send.
///
/// # Arguments
/// * `message` - message string (should match message.rs/ExtendedMessage)
/// * `handler` - function that will be executed, when the protocol and the step name matches the
///               message type
///
/// # Returns
/// * `ProtocolStep` - The new protocol step, that can be pushed to a protocol steps vec.
pub fn generate_send_step(name: &str, handler: fn(message: &str) -> StepResult) -> ProtocolStep {
    return ProtocolStep {
        direction: MessageDirection::SEND,
        name: String::from(name),
        handler,
    };
}

/// Shorthand generator for a protocol step, with direction receive.
///
/// # Arguments
/// * `message` - message string (should match message.rs/ExtendedMessage)
/// * `handler` - function that will be executed, when the protocol and the step name matches the
///               message type
///
/// # Returns
/// * `ProtocolStep` - The new protocol step, that can be pushed to a protocol steps vec.
pub fn generate_receive_step(name: &str, handler: fn(message: &str) -> StepResult) -> ProtocolStep {
    return ProtocolStep {
        direction: MessageDirection::RECEIVE,
        name: String::from(name),
        handler,
    };
}

/// Shorthand generator for a protocol step output, with encrypt flag set to true.
///
/// # Arguments
/// * `message`  - message string (should match message.rs/ExtendedMessage)
/// * `metadata` - general protocol step specific data
///
/// # Returns
/// * `StepResult` - Result that will be populated to the vade_didcomm
pub fn generate_step_output(message: &str, metadata: &str) -> StepResult {
    return Ok(StepOutput {
        encrypt: true,
        message: String::from(message),
        metadata: String::from(metadata),
    });
}

/// Shorthand generator for a protocol step output, with encrypt flag set to false.
///
/// # Arguments
/// * `message`  - message string (should match message.rs/ExtendedMessage)
/// * `metadata` - general protocol step specific data
///
/// # Returns
/// * `StepResult` - Result that will be populated to the vade_didcomm
#[allow(dead_code)]
pub fn generate_step_output_decrypted(message: &str, metadata: &str) -> StepResult {
    return Ok(StepOutput {
        encrypt: false,
        message: String::from(message),
        metadata: String::from(metadata),
    });
}
