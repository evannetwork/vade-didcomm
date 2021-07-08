use crate::{Message, utils::SyncResult};

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
    pub handler: fn(message: &mut Message) -> StepResult,
    pub name: String,
}

pub struct StepOutput {
    pub encrypt: bool,
    pub metadata: String,
}

pub type StepResult = SyncResult<StepOutput>;

pub fn send_step(
    name: &str,
    handler: fn(message: &mut Message) -> StepResult,
) -> ProtocolStep {
    return ProtocolStep {
        direction: Direction::SEND,
        name: String::from(name),
        handler,
    };
}

pub fn receive_step(
    name: &str,
    handler: fn(message: &mut Message) -> StepResult,
) -> ProtocolStep {
    return ProtocolStep {
        direction: Direction::RECEIVE,
        name: String::from(name),
        handler,
    };
}
