use crate::{
    datatypes::{ExtendedMessage, ProblemReport},
    protocols::protocol::{generate_step_output, StepResult},
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/problem-report`
pub fn send_problem_report(message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation data not provided.")?,
    )?;
    let problem_report: ProblemReport = serde_json::from_str(&data)?;

    generate_step_output(&serde_json::to_string(&problem_report)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/problem-report`
pub fn receive_problem_report(message: &str) -> StepResult {
    generate_step_output(message, "{}")
}