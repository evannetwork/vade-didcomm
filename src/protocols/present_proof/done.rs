#[cfg(feature = "state_storage")]
use crate::protocols::present_proof::{
    datatypes::{State, UserType},
    presentation::{get_current_state, save_state},
};
use crate::{
    datatypes::MessageWithBody,
    protocols::{
        present_proof::datatypes::AckData,
        protocol::{generate_step_output, StepResult},
    },
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn send_presentation_ack(_options: &str, message: &str) -> StepResult {
    let ack_message: MessageWithBody<AckData> = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let thid = ack_message
                .thid
                .as_ref()
                .ok_or("Thread id can't be empty")?;

            let current_state: State = get_current_state(thid, &UserType::Verifier)?.parse()?;

            match current_state {
                State::PresentationReceived => {
                    save_state(thid, &State::Acknowledged, &UserType::Verifier)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::Acknowledged
                    )))
                }
            }
        } else { }
    }

    generate_step_output(&serde_json::to_string(&ack_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn receive_presentation_ack(_options: &str, message: &str) -> StepResult {
    #[allow(unused_variables)] // may not be used afterwards but call is needed to validate input
    let ack_message: MessageWithBody<AckData> = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let thid = ack_message.thid.ok_or("Thread id can't be empty")?;

            let current_state: State = get_current_state(&thid, &UserType::Prover)?.parse()?;

            match current_state {
                State::PresentationSent => {
                    save_state(&thid, &State::Acknowledged, &UserType::Prover)?;
                }
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::Acknowledged
                    )))
                }
            }
        } else { }
    }

    generate_step_output(message, "{}")
}
