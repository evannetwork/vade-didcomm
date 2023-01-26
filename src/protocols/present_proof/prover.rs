#[cfg(feature = "state_storage")]
use super::datatypes::PROPOSAL_PROTOCOL_URL;
#[cfg(feature = "state_storage")]
use crate::{
    datatypes::HasFromAndTo,
    protocols::present_proof::{
        datatypes::{State, UserType},
        presentation::{get_current_state, save_presentation, save_state},
    },
};
use crate::{
    datatypes::MessageWithBody,
    protocols::{
        present_proof::datatypes::{PresentationData, ProposalData, RequestData},
        protocol::{generate_step_output, StepResult},
    },
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/presentation`
pub fn send_presentation(_options: &str, message: &str) -> StepResult {
    let presentation_message: MessageWithBody<PresentationData> = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let presentation_data = presentation_message
                .body
                .as_ref()
                .ok_or("missing presentation data in body")?;
            let from_to = presentation_message.get_from_to()?;
            let thid = presentation_message
                .thid
                .as_ref()
                .ok_or("Thread id can't be empty")?;

            let current_state: State = get_current_state(thid, &UserType::Prover)?.parse()?;
            match current_state {
                State::PresentationRequestReceived => {
                    save_state(thid, &State::PresentationSent, &UserType::Prover)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::PresentationSent
                    )))
                }
            };

            save_presentation(
                &from_to.from,
                &from_to.to,
                thid,
                &serde_json::to_string(&presentation_data)?,
                &State::PresentationSent,
            )?;
        } else { }
    }

    generate_step_output(&serde_json::to_string(&presentation_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/request_presentation`
pub fn receive_request_presentation(_options: &str, message: &str) -> StepResult {
    #[allow(unused_variables)] // may not be used afterwards but call is needed to validate input
    let request_message: MessageWithBody<RequestData> = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
        let request_data = request_message
            .body
            .as_ref()
            .ok_or("missing request data in body")?;
        let from_to = request_message.get_from_to()?;
        let thid = request_message
            .thid
            .to_owned()
            .ok_or("Thread id can't be empty")?;

        let current_state: State = get_current_state(&thid, &UserType::Prover)?.parse()?;
        match current_state {
            State::PresentationProposed | State::Unknown => save_state(
                &thid,
                &State::PresentationRequestReceived,
                &UserType::Prover,
            )?,
            _ => {
                return Err(Box::from(format!(
                    "Error while processing step: State from {} to {} not allowed",
                    current_state,
                    State::PresentationRequestReceived
                )))
            }
        };

        save_presentation(
            &from_to.to,
            &from_to.from,
            &thid,
            &serde_json::to_string(&request_data)?,
            &State::PresentationRequestReceived,
        )?;
    } else { }
    }

    generate_step_output(message, "{}")
}

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/propose-presentation`
pub fn send_propose_presentation(_options: &str, message: &str) -> StepResult {
    let proposal_message: MessageWithBody<ProposalData> = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let proposal_data = proposal_message
                .body
                .as_ref()
                .ok_or("missing proposal data in body")?;
            if proposal_data.presentation_proposal.r#type != PROPOSAL_PROTOCOL_URL {
                return Err(Box::from(format!(
                    r#"invalid type in proposal: "{}", must be "{}"#,
                    &proposal_data.presentation_proposal.r#type, PROPOSAL_PROTOCOL_URL
                )));
            }
            let from_to = proposal_message.get_from_to()?;
            let thid = proposal_message
                .thid
                .as_ref()
                .ok_or("Thread id can't be empty")?;

            let current_state: State = get_current_state(thid, &UserType::Prover)?.parse()?;

            match current_state {
                State::PresentationRequestReceived | State::Unknown => {
                    save_state(thid, &State::PresentationProposed, &UserType::Prover)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::PresentationProposed
                    )))
                }
            };

            save_presentation(
                &from_to.from,
                &from_to.to,
                thid,
                &serde_json::to_string(&proposal_data)?,
                &State::PresentationProposed,
            )?;
        } else { }
    }

    generate_step_output(&serde_json::to_string(&proposal_message)?, "{}")
}
