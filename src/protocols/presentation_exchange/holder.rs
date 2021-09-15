use crate::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody},
    get_from_to_from_message,
    protocols::presentation_exchange::presentation_exchange::{get_current_state, save_presentation_exchange, save_state},
    protocols::presentation_exchange::datatypes::{PresentationExchangeData, State, UserType},
    protocols::protocol::{generate_step_output, StepResult},
};

use super::helper::{
    get_presentation_exchange_info_from_message,
    get_presentation_exchange_message,
    PresentationExchangeType,
};

/// Protocol handler for direction: `send`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/propose-presentation`
pub fn send_propose_presentation(message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(base_message)?;

    let data =
        &serde_json::to_string(&parsed_message.body.ok_or("Presentation exchange data not provided.")?)?;
    let presentation_exchange_data: PresentationExchangeData = serde_json::from_str(&data)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty.")?;

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

    let request_message = get_presentation_exchange_message(
        PresentationExchangeType::ProposePresentation,
        &exchange_info.from,
        &exchange_info.to,
        presentation_exchange_data.clone(),
        &thid,
    )?;

    save_presentation_exchange(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&presentation_exchange_data)?,
        &State::SendProposePresentation,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")

}

/// Protocol handler for direction: `receive`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/request-presentation`
pub fn receive_request_presentation(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<PresentationExchangeData> = serde_json::from_str(message)?;

    let base_message: BaseMessage = BaseMessage {
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
    let base_info = get_from_to_from_message(base_message)?;
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

    generate_step_output(message, "{}")
}

/// Protocol handler for direction: `send`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/presentation`
pub fn send_presentation(message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(base_message)?;

    let data =
        &serde_json::to_string(&parsed_message.body.ok_or("Presentation exchagne data not provided.")?)?;
    let presentation_exchange_data: PresentationExchangeData = serde_json::from_str(&data)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty.")?;

    let current_state: State = get_current_state(&thid, &UserType::Holder)?.parse()?;

    match current_state {
        State::ReceivePresentatonRequest | State::Unknown => {
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

    let request_message = get_presentation_exchange_message(
        PresentationExchangeType::Presentation,
        &exchange_info.from,
        &exchange_info.to,
        presentation_exchange_data.clone(),
        &thid,
    )?;

    save_presentation_exchange(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&presentation_exchange_data)?,
        &State::SendPresentation,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")

}
