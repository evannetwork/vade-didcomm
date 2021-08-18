use crate::{
    datatypes::{ExtendedMessage, ProblemReport, State},
    presentation::{get_current_state, save_state},
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
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;
    let current_state: State = get_current_state(&thid)?.parse()?;

    let result = match current_state {
        State::PresentationRequested => save_state(&thid, &State::ProblemReported),
        State::PresentationRequestReceived => save_state(&thid, &State::ProblemReported),
        State::PresentationSent => save_state(&thid, &State::ProblemReported),
        State::PresentationReceived => save_state(&thid, &State::ProblemReported),
        State::PresentationProposalReceived => save_state(&thid, &State::ProblemReported),
        State::PresentationProposed => save_state(&thid, &State::ProblemReported),
        _ => Err(Box::from(format!(
            "State from {} to {} not allowed",
            current_state,
            State::ProblemReported
        ))),
    };

    match result {
        Ok(_) => {}
        Err(err) => panic!("Error while processing step: {:?}", err),
    }

    generate_step_output(&serde_json::to_string(&problem_report)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/problem-report`
pub fn receive_problem_report(message: &str) -> StepResult {
    let parsed_message: ProblemReport = serde_json::from_str(&message)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;
    let current_state: State = get_current_state(&thid)?.parse()?;

    let result = match current_state {
        State::PresentationRequested => save_state(&thid, &State::ProblemReported),
        State::PresentationRequestReceived => save_state(&thid, &State::ProblemReported),
        State::PresentationSent => save_state(&thid, &State::ProblemReported),
        State::PresentationReceived => save_state(&thid, &State::ProblemReported),
        State::PresentationProposalReceived => save_state(&thid, &State::ProblemReported),
        State::PresentationProposed => save_state(&thid, &State::ProblemReported),
        _ => Err(Box::from(format!(
            "State from {} to {} not allowed",
            current_state,
            State::ProblemReported
        ))),
    };

    match result {
        Ok(_) => {}
        Err(err) => panic!("Error while processing step: {:?}", err),
    }
    generate_step_output(message, "{}")
}
