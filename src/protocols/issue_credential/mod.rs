mod credential;
pub mod datatypes;
mod done;
mod helper;
mod holder;
mod issuer;
mod problem_report;

use crate::protocols::{
    issue_credential::{
        datatypes::ISSUE_CREDENTIAL_PROTOCOL_URL,
        done::{receive_credential_ack, send_credential_ack},
        holder::{
            receive_issue_credential, receive_offer_credential, send_propose_credential,
            send_request_credential,
        },
        issuer::{
            receive_propose_credential, receive_request_credential, send_issue_credential,
            send_offer_credential,
        },
        problem_report::{receive_problem_report, send_problem_report},
    },
    protocol::{generate_receive_step, generate_send_step, Protocol},
};

/// Creates the issue_credential protocol, containing step handler functions mapped to their according step.
///
/// # Returns
/// * `Protocol` - the new Issue credential protocol handler
pub fn generate_issue_credential_protocol() -> Protocol {
    Protocol {
        name: String::from(ISSUE_CREDENTIAL_PROTOCOL_URL),
        steps: vec![
            generate_send_step("propose-credential", send_propose_credential),
            generate_receive_step("propose-credential", receive_propose_credential),
            generate_send_step("offer-credential", send_offer_credential),
            generate_receive_step("offer-credential", receive_offer_credential),
            generate_send_step("request-credential", send_request_credential),
            generate_receive_step("request-credential", receive_request_credential),
            generate_send_step("issue-credential", send_issue_credential),
            generate_receive_step("issue-credential", receive_issue_credential),
            generate_send_step("ack", send_credential_ack),
            generate_receive_step("ack", receive_credential_ack),
            generate_send_step("problem-report", send_problem_report),
            generate_receive_step("problem-report", receive_problem_report),
        ],
    }
}
