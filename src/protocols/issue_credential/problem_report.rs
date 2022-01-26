use crate::protocols::{
    issue_credential::{
        credential::{get_current_state, save_state},
        datatypes::{ProblemReport, State, UserType},
    },
    protocol::{generate_step_output, StepResult},
};

/// Protocol handler for direction: `send`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/problem-report`
pub fn send_problem_report(_options: &str, message: &str) -> StepResult {
    let problem_report: ProblemReport = serde_json::from_str(message)?;
    let problem_report_data = &problem_report.body;
    let thid = problem_report
        .thid
        .as_ref()
        .ok_or("Thread id can't be empty")?;
    let current_state: State = get_current_state(thid, &problem_report_data.user_type)?.parse()?;

    match current_state {
        State::ReceiveProposeCredential | State::ReceiveOfferCredential => save_state(
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

    generate_step_output(&serde_json::to_string(&problem_report)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/problem-report`
pub fn receive_problem_report(_options: &str, message: &str) -> StepResult {
    let problem_report: ProblemReport = serde_json::from_str(message)?;
    let thid = problem_report.thid.ok_or("Thread id can't be empty")?;

    // flip sides to get current users type
    let current_user_type = match &problem_report.body.user_type {
        UserType::Issuer => UserType::Holder,
        UserType::Holder => UserType::Issuer,
        _ => {
            return Err(Box::from(format!(
                "invalid user type for problem report: {}",
                &problem_report.body.user_type
            )))
        }
    };
    let current_state: State = get_current_state(&thid, &current_user_type)?.parse()?;

    match current_state {
        State::SendProposeCredential | State::SendOfferCredential => {
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

    generate_step_output(message, "{}")
}
