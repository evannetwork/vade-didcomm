use crate::{
    datatypes::ExtendedMessage,
    protocols::issue_credential::credential::{get_current_state, save_state},
    protocols::issue_credential::datatypes::{ProblemReport, State},
    protocols::protocol::{generate_step_output, StepResult},
};

/// Protocol handler for direction: `send`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/problem-report`
pub fn send_problem_report(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let data =
        &serde_json::to_string(&parsed_message.body.ok_or("Credential data not provided.")?)?;
    let problem_report: ProblemReport = serde_json::from_str(data)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;
    let current_state: State = get_current_state(&thid, &problem_report.user_type)?.parse()?;

    match current_state {
        State::ReceiveProposeCredential | State::ReceiveOfferCredential => {
            save_state(&thid, &State::ProblemReported, &problem_report.user_type)?
        }
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
    let parsed_message: ProblemReport = serde_json::from_str(message)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;
    let current_state: State = get_current_state(&thid, &parsed_message.user_type)?.parse()?;

    match current_state {
        State::SendProposeCredential | State::SendOfferCredential => {
            save_state(&thid, &State::ProblemReported, &parsed_message.user_type)?
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
