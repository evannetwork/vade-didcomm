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

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/request-presentation`
pub fn send_request_presentation(_options: &str, message: &str) -> StepResult {
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
            .as_ref()
            .ok_or("Thread id can't be empty")?;

        let current_state: State = get_current_state(thid, &UserType::Verifier)?.parse()?;
        match current_state {
            State::PresentationProposalReceived | State::Unknown => {
                save_state(thid, &State::PresentationRequested, &UserType::Verifier)?
            }
            _ => {
                return Err(Box::from(format!(
                    "Error while processing step: State from {} to {} not allowed",
                    current_state,
                    State::PresentationRequested
                )))
            }
        };

        save_presentation(
            &from_to.from,
            &from_to.to,
            thid,
            &serde_json::to_string(&request_data)?,
            &State::PresentationRequested,
        )?;
    } else { }
    }

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/presentation`
pub fn receive_presentation(_options: &str, message: &str) -> StepResult {
    #[allow(unused_variables)] // may not be used afterwards but call is needed to validate input
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

            let current_state: State = get_current_state(thid, &UserType::Verifier)?.parse()?;
            match current_state {
                State::PresentationRequested => {
                    save_state(thid, &State::PresentationReceived, &UserType::Verifier)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::PresentationReceived
                    )))
                }
            };

            save_presentation(
                &from_to.from,
                &from_to.to,
                thid,
                &serde_json::to_string(&presentation_data)?,
                &State::PresentationReceived,
            )?;
        } else { }
    }

    generate_step_output(message, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/propose-presentation`
pub fn receive_propose_presentation(_options: &str, message: &str) -> StepResult {
    #[allow(unused_variables)] // may not be used afterwards but call is needed to validate input
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
                .to_owned()
                .ok_or("Thread id can't be empty")?;

            let current_state: State = get_current_state(thid, &UserType::Verifier)?.parse()?;
            match current_state {
                State::PresentationRequested | State::Unknown => save_state(
                    thid,
                    &State::PresentationProposalReceived,
                    &UserType::Verifier,
                )?,
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::PresentationProposalReceived
                    )))
                }
            };

            save_presentation(
                &from_to.from,
                &from_to.to,
                thid,
                &serde_json::to_string(&proposal_data)?,
                &State::PresentationProposalReceived,
            )?;
        } else { }
    }

    generate_step_output(message, "{}")
}
