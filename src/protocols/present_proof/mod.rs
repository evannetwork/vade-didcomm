pub(crate) mod done;
pub(crate) mod helper;
pub(crate) mod prover;
pub(crate) mod verifier;
pub(crate) mod problem_report;

use crate::{
    protocols::{
        present_proof::{
            done::{receive_presentation_ack, send_presentation_ack},
            prover::{receive_request_presentation, send_presentation, send_propose_presentation},
            problem_report::{send_problem_report,receive_problem_report},
            verifier::{
                receive_presentation, receive_propose_presentation, send_request_presentation,
            },
        },
        protocol::{generate_receive_step, generate_send_step, Protocol},
    },
};
use helper::PRESENT_PROOF_PROTOCOL_URL;

/// Creates the present_proof protocol, containing step handler functions mapped to their according step.
///
/// # Returns
/// * `Protocol` - the new Present proof protocol handler
pub fn generate_present_proof_protocol() -> Protocol {
    Protocol {
        name: String::from(PRESENT_PROOF_PROTOCOL_URL),
        steps: vec![
            generate_send_step("request-presentation", send_request_presentation),
            generate_receive_step("presentation", receive_presentation),
            generate_receive_step("propose-presentation", receive_propose_presentation),
            generate_receive_step("request-presentation", receive_request_presentation),
            generate_send_step("presentation", send_presentation),
            generate_send_step("propose-presentation", send_propose_presentation),
            generate_send_step("ack", send_presentation_ack),
            generate_receive_step("ack", receive_presentation_ack),
            generate_send_step("problem-report", send_problem_report),
            generate_receive_step("problem-report", receive_problem_report),
        ],
    }
}
