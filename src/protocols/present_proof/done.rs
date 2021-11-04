use crate::{
    datatypes::ExtendedMessage,
    protocols::{
        present_proof::{
            datatypes::{Ack, State},
            presentation::{get_current_state, save_state},
        },
        protocol::{generate_step_output, StepResult},
    },
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn send_presentation_ack(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation data not provided.")?,
    )?;
    let ack: Ack = serde_json::from_str(data)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &ack.user_type)?.parse()?;

    match current_state {
        State::PresentationReceived => save_state(&thid, &State::Acknowledged, &ack.user_type)?,
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::Acknowledged
            )))
        }
    }

    generate_step_output(&serde_json::to_string(&ack)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn receive_presentation_ack(_options: &str, message: &str) -> StepResult {
    let parsed_message: Ack = serde_json::from_str(message)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &parsed_message.user_type)?.parse()?;

    match current_state {
        State::PresentationSent => {
            save_state(&thid, &State::Acknowledged, &parsed_message.user_type)?;
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::Acknowledged
            )))
        }
    }

    generate_step_output(message, "{}")
}
