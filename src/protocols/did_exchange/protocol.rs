use crate::{Protocol, receive_step, request::{receive_request, send_request}, send_step};

pub const DID_EXCHANGE_PROTOCOL_URL: &str = "https://didcomm.org/didexchange/1.0";

macro_rules! sf {
    ( $var:expr ) => ( String::from($var) );
}

pub fn get_did_exchange_protocol() -> Protocol {
    let mut protocol = Protocol {
        name: sf!(DID_EXCHANGE_PROTOCOL_URL),
        steps: Vec::new(),
    };

    protocol.steps.push(send_step("request", send_request));
    protocol.steps.push(receive_step("request", receive_request));

    return protocol;
}
