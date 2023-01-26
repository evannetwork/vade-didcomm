use std::collections::HashMap;

#[cfg(feature = "state_storage")]
use super::helper::get_presentation_exchange_info_from_message;
use super::helper::{get_presentation_exchange_message, PresentationExchangeType};
#[cfg(feature = "state_storage")]
use crate::protocols::presentation_exchange::{
    datatypes::{State, UserType},
    presentation_exchange_data::{get_current_state, save_presentation_exchange, save_state},
};
use crate::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody},
    get_from_to_from_message,
    protocols::{
        presentation_exchange::datatypes::PresentationExchangeData,
        protocol::{generate_step_output, StepResult},
    },
};

/// Protocol handler for direction: `send`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/propose-presentation`
pub fn send_propose_presentation(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        body: HashMap::new(),
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(&base_message)?;

    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation exchange data not provided.")?,
    )?;
    let presentation_exchange_data: PresentationExchangeData = serde_json::from_str(data)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty.")?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let current_state: State = get_current_state(&thid, &UserType::Holder)?.parse()?;

            match current_state {
                State::ReceivePresentatonRequest | State::Unknown => {
                    save_state(&thid, &State::SendProposePresentation, &UserType::Holder)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::SendProposePresentation
                    )))
                }
            };
        } else { }
    }

    let request_message = get_presentation_exchange_message(
        PresentationExchangeType::ProposePresentation,
        &exchange_info.from,
        &exchange_info.to,
        presentation_exchange_data.clone(),
        &thid,
    )?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            save_presentation_exchange(
                &exchange_info.from,
                &exchange_info.to,
                &thid,
                &serde_json::to_string(&presentation_exchange_data)?,
                &State::SendProposePresentation,
            )?;
        } else { }
    }

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/request-presentation`
pub fn receive_request_presentation(_options: &str, message: &str) -> StepResult {
    #[allow(unused_variables)] // may not be used afterwards but call is needed to validate input
    let parsed_message: MessageWithBody<PresentationExchangeData> = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let base_message: BaseMessage = BaseMessage {
                body: HashMap::new(),
                from: parsed_message.from.clone(),
                r#type: parsed_message.r#type.clone(),
                to: Some(
                    parsed_message
                        .to
                        .clone()
                        .ok_or("To DID not provided")?
                        .to_vec(),
                ),
            };
            let thid = parsed_message
                .thid
                .to_owned()
                .ok_or("Thread id can't be empty")?;

            let exchange_info = get_presentation_exchange_info_from_message(parsed_message)?;
            let base_info = get_from_to_from_message(&base_message)?;
            let presentation_exchange_data = exchange_info
                .presentation_exchange_data
                .ok_or("Presentation exchange data not provided.")?;

            let current_state: State = get_current_state(&thid, &UserType::Holder)?.parse()?;

            match current_state {
                State::Unknown | State::SendProposePresentation => {
                    save_state(&thid, &State::ReceivePresentatonRequest, &UserType::Holder)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::ReceivePresentatonRequest
                    )))
                }
            };

            save_presentation_exchange(
                &base_info.to,
                &base_info.from,
                &thid,
                &serde_json::to_string(&presentation_exchange_data)?,
                &State::ReceivePresentatonRequest,
            )?;
        } else { }
    }

    generate_step_output(message, "{}")
}

/// Protocol handler for direction: `send`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/presentation`
pub fn send_presentation(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        body: HashMap::new(),
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(&base_message)?;

    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation exchagne data not provided.")?,
    )?;
    let presentation_exchange_data: PresentationExchangeData = serde_json::from_str(data)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty.")?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let current_state: State = get_current_state(&thid, &UserType::Holder)?.parse()?;

            match current_state {
                State::ReceivePresentatonRequest => {
                    save_state(&thid, &State::SendPresentation, &UserType::Holder)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "Error while processing step: State from {} to {} not allowed",
                        current_state,
                        State::SendPresentation
                    )))
                }
            };
      } else { }
    }

    let request_message = get_presentation_exchange_message(
        PresentationExchangeType::Presentation,
        &exchange_info.from,
        &exchange_info.to,
        presentation_exchange_data.clone(),
        &thid,
    )?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            save_presentation_exchange(
                &exchange_info.from,
                &exchange_info.to,
                &thid,
                &serde_json::to_string(&presentation_exchange_data)?,
                &State::SendPresentation,
            )?;
        } else { }
    }

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}
