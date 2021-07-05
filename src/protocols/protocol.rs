use crate::Message;

pub struct ProtocolConfig{
    pub name: String,
    pub handler: fn(message: &mut Message, encrypt: &mut bool),
}

pub struct Protocol {
    pub name: String,
    pub steps: Vec<ProtocolConfig>,
}
