use crate::{
    protocols::did_exchange::{
        complete::{receive_complete, send_complete},
        request::{receive_request, send_request},
        response::{receive_response, send_response},
    },
    receive_step, send_step, Protocol,
};

pub const DID_EXCHANGE_PROTOCOL_URL: &str = "https://didcomm.org/didexchange/1.0";

macro_rules! sf {
    ( $var:expr ) => {
        String::from($var)
    };
}

/// Creates a new did_exchange protocol and maps the specific step handler functions.
///
/// # Returns
/// * `Protocol` - the new did exchange protocol handler
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
    protocol.steps.push(send_step("complete", send_complete));
    protocol
        .steps
        .push(receive_step("complete", receive_complete));

    return protocol;
}
