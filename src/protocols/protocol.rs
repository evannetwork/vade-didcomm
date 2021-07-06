use crate::{Message, utils::SyncResult};

pub enum Direction {
    SEND,
    RECEIVE,
}

pub struct ProtocolStep {
    pub direction: Direction,
    pub handler: fn(message: &mut Message, encrypt: &mut bool) -> SyncResult<String>,
    pub name: String,
}

pub struct Protocol {
    pub name: String,
    pub steps: Vec<ProtocolStep>,
}

pub fn send_step(
    name: &str,
    handler: fn(message: &mut Message, encrypt: &mut bool) -> SyncResult<String>,
) -> ProtocolStep {
    return ProtocolStep {
        direction: Direction::SEND,
        name: String::from(name),
        handler: handler,
    };
}

pub fn receive_step(
    name: &str,
    handler: fn(message: &mut Message, encrypt: &mut bool) -> SyncResult<String>,
) -> ProtocolStep {
    return ProtocolStep {
        direction: Direction::RECEIVE,
        name: String::from(name),
        handler: handler,
    };
}
