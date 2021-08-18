use crate::{
    datatypes::{Ack, ExtendedMessage, State},
    presentation::{get_current_state, save_state},
    protocols::protocol::{generate_step_output, StepResult},
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn send_presentation_ack(message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation data not provided.")?,
    )?;
    let ack: Ack = serde_json::from_str(&data)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &ack.user_type)?.parse()?;

    let result = match current_state {
        State::PresentationReceived => save_state(&thid, &State::Acknowledged, &ack.user_type),
        _ => Err(Box::from(format!(
            "State from {} to {} not allowed",
            current_state,
            State::Acknowledged
        ))),
    };

    match result {
        Ok(_) => {}
        Err(err) => panic!("Error while processing step: {:?}", err),
    }

    generate_step_output(&serde_json::to_string(&ack)?, "{}")
}
/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn receive_presentation_ack(message: &str) -> StepResult {
    let parsed_message: Ack = serde_json::from_str(&message)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &parsed_message.user_type)?.parse()?;

    let result = match current_state {
        State::PresentationSent => {
            save_state(&thid, &State::Acknowledged, &parsed_message.user_type)
        }
        State::Acknowledged => Ok(()),
        _ => Err(Box::from(format!(
            "State from {} to {} not allowed",
            current_state,
            State::Acknowledged
        ))),
    };

    match result {
        Ok(_) => {}
        Err(err) => panic!("Error while processing step: {:?}", err),
    }
    generate_step_output(message, "{}")
}
