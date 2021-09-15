pub mod datatypes;
mod helper;
mod holder;
mod presentation_exchange_data;
mod verifier;

use crate::protocols::{
    presentation_exchange::{
        datatypes::PRESENTATION_EXCHANGE_PROTOCOL_URI,
        holder::{receive_request_presentation, send_presentation, send_propose_presentation},
        verifier::{receive_presentation, receive_propose_presentation, send_request_presentation},
    },
    protocol::{generate_receive_step, generate_send_step, Protocol},
};

/// Creates the presentation_exchange protocol, containing step handler functions mapped to their according step.
///
/// # Returns
/// * `Protocol` - the new Presentation exchange protocol handler
pub fn generate_issue_credential_protocol() -> Protocol {
    Protocol {
        name: String::from(PRESENTATION_EXCHANGE_PROTOCOL_URI),
        steps: vec![
            generate_send_step("request-presentation", send_request_presentation),
            generate_receive_step("request-presentation", receive_request_presentation),
            generate_send_step("propose-presentation", send_propose_presentation),
            generate_receive_step("propose-presentation", receive_propose_presentation),
            generate_send_step("presentation", send_presentation),
            generate_receive_step("presentation", receive_presentation),
        ],
    }
}
