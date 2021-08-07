pub(crate) mod done;
pub(crate) mod helper;
pub(crate) mod verifier;
pub(crate) mod prover;

use crate::{
    datatypes::DID_EXCHANGE_PROTOCOL_URL,
    protocols::{
        present_proof::{
            done::{receive_presentation_ack, send_presentation_ack},
            verifier::{send_request_presentation, receive_presentation, receive_propose_presentation},
            prover::{send_presentation, receive_request_presentation, send_propose_presentation},
        },
        protocol::{generate_receive_step, generate_send_step, Protocol},
    },
};

/// Creates a new present_proof protocol and maps the specific step handler functions.
///
/// # Returns
/// * `Protocol` - the new Present proof protocol handler
pub fn generate_present_proof_protocol() -> Protocol {
    let protocol = Protocol {
        name: String::from(DID_EXCHANGE_PROTOCOL_URL),
        steps: vec![
            generate_send_step("request-presentation", send_request_presentation),
            generate_receive_step("presentation", receive_presentation),
            generate_receive_step("propose-presentation", receive_propose_presentation),
            generate_receive_step("request-presentation", receive_request_presentation),
            generate_send_step("presentation", send_presentation),
            generate_send_step("propose-presentation", send_propose_presentation),
            generate_send_step("ack", send_presentation_ack),
            generate_receive_step("ack", receive_presentation_ack),
        ],
    };

    return protocol;
}
