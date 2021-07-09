use crate::{utils::SyncResult};

#[derive(PartialEq)]
pub enum Direction {
    SEND,
    RECEIVE,
}

pub struct Protocol {
    pub name: String,
    pub steps: Vec<ProtocolStep>,
}

pub struct ProtocolStep {
    pub direction: Direction,
    pub handler: fn(message: &str) -> StepResult,
    pub name: String,
}

pub struct StepOutput {
    pub encrypt: bool,
    pub metadata: String,
    pub message: String,
}

pub type StepResult = SyncResult<StepOutput>;

pub fn send_step(
    name: &str,
    handler: fn(message: &str) -> StepResult,
) -> ProtocolStep {
    return ProtocolStep {
        direction: Direction::SEND,
        name: String::from(name),
        handler,
    };
}

pub fn receive_step(
    name: &str,
    handler: fn(message: &str) -> StepResult,
) -> ProtocolStep {
    return ProtocolStep {
        direction: Direction::RECEIVE,
        name: String::from(name),
        handler,
    };
}

pub fn get_step_output(message: &str, metadata: &str) -> StepResult {
    return Ok(StepOutput {
        encrypt: true,
        message: String::from(message),
        metadata: String::from(metadata),
    });
}

pub fn get_step_output_decrypted(message: &str, metadata: &str) -> StepResult {
    return Ok(StepOutput {
        encrypt: false,
        message: String::from(message),
        metadata: String::from(metadata),
    });
}
