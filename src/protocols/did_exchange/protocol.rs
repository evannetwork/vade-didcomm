use crate::{
    receive_step,
    request::{receive_request, send_request},
    response::{receive_response, send_response},
    send_step, Protocol,
};

pub const DID_EXCHANGE_PROTOCOL_URL: &str = "https://didcomm.org/didexchange/1.0";

macro_rules! sf {
    ( $var:expr ) => {
        String::from($var)
    };
}

pub fn get_did_exchange_protocol() -> Protocol {
    let mut protocol = Protocol {
        name: sf!(DID_EXCHANGE_PROTOCOL_URL),
        steps: Vec::new(),
    };

    protocol.steps.push(send_step("request", send_request));
    protocol
        .steps
        .push(receive_step("request", receive_request));
    protocol.steps.push(send_step("response", send_response));
    protocol
        .steps
        .push(receive_step("response", receive_response));

    return protocol;
}
