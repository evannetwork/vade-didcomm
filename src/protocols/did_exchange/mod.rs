pub(crate) mod complete;
pub mod datatypes;
mod did_exchange;
pub(crate) mod helper;
mod problem_report;
pub(crate) mod request;
pub(crate) mod response;

use crate::protocols::{
    did_exchange::{
        complete::{receive_complete, send_complete},
        datatypes::DID_EXCHANGE_PROTOCOL_URL,
        problem_report::{receive_problem_report, send_problem_report},
        request::{receive_request, send_request},
        response::{receive_response, send_response},
    },
    protocol::{generate_receive_step, generate_send_step, Protocol},
};

/// Creates a new did_exchange protocol and maps the specific step handler functions.
///
/// # Returns
/// * `Protocol` - the new DID exchange protocol handler
pub(crate) fn generate_did_exchange_protocol() -> Protocol {
    Protocol {
        name: String::from(DID_EXCHANGE_PROTOCOL_URL),
        steps: vec![
            generate_send_step("request", send_request),
            generate_receive_step("request", receive_request),
            generate_send_step("response", send_response),
            generate_receive_step("response", receive_response),
            generate_send_step("complete", send_complete),
            generate_receive_step("complete", receive_complete),
            generate_send_step("problem-report", send_problem_report),
            generate_receive_step("problem-report", receive_problem_report),
        ],
    }
}

pub use helper::DidExchangeOptions;
