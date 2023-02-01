#[cfg(feature = "state_storage")]
use crate::protocols::present_proof::{
    datatypes::{State, UserType},
    presentation::{get_current_state, save_state},
};
use crate::{
    datatypes::MessageWithBody,
    protocols::{
        present_proof::datatypes::ProblemReportData,
        protocol::{generate_step_output, StepResult},
    },
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/problem-report`
pub fn send_problem_report(_options: &str, message: &str) -> StepResult {
    let problem_report_message: MessageWithBody<ProblemReportData> = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let problem_report_data = problem_report_message
                .body
                .as_ref()
                .ok_or("missing problem report data in body")?;
            let thid = &problem_report_message
                .thid
                .as_ref()
                .ok_or("Thread id can't be empty")?;
            let current_state: State = get_current_state(thid, &problem_report_data.user_type)?
                .parse()?;

            match current_state {
                State::PresentationRequested
                | State::PresentationRequestReceived
                | State::PresentationSent
                | State::PresentationReceived
                | State::PresentationProposalReceived
                | State::PresentationProposed => save_state(
                    thid,
                    &State::ProblemReported,
                    &problem_report_data.user_type,
                )?,
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::ProblemReported
                    )))
                }
            };
        } else { }
    }

    generate_step_output(&serde_json::to_string(&problem_report_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/problem-report`
pub fn receive_problem_report(_options: &str, message: &str) -> StepResult {
    #[allow(unused_variables)] // may not be used afterwards but call is needed to validate input
    let problem_report_message: MessageWithBody<ProblemReportData> = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
    if #[cfg(feature = "state_storage")] {
        let problem_report_data = problem_report_message
            .body
            .as_ref()
            .ok_or("missing problem report data in body")?;
        let thid = problem_report_message
            .thid
            .ok_or("Thread id can't be empty")?;

        // flip sides to get current users type
        let current_user_type = match &problem_report_data.user_type {
            UserType::Prover => UserType::Verifier,
            UserType::Verifier => UserType::Prover,
            _ => {
                return Err(Box::from(format!(
                    "invalid user type for problem report: {}",
                    &problem_report_data.user_type
                )))
            }
        };
        let current_state: State = get_current_state(&thid, &current_user_type)?.parse()?;

        match current_state {
            State::PresentationRequested
            | State::PresentationRequestReceived
            | State::PresentationSent
            | State::PresentationReceived
            | State::PresentationProposalReceived
            | State::PresentationProposed => {
                save_state(&thid, &State::ProblemReported, &current_user_type)?
            }
            _ => {
                return Err(Box::from(format!(
                    "Error while processing step: State from {} to {} not allowed",
                    current_state,
                    State::ProblemReported
                )))
            }
        };
    } else { }
    }

    generate_step_output(message, "{}")
}
