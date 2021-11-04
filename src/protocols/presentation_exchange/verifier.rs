use super::helper::{
    get_presentation_exchange_info_from_message,
    get_presentation_exchange_message,
    validate_presentation_against_credentials,
    PresentationExchangeType,
};
use crate::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody},
    get_from_to_from_message,
    protocols::{
        presentation_exchange::{
            datatypes::{PresentationExchangeData, State, UserType},
            presentation_exchange_data::{
                get_current_state,
                get_presentation_exchange,
                save_presentation_exchange,
                save_state,
            },
        },
        protocol::{generate_step_output, StepResult},
    },
};

/// Protocol handler for direction: `send`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/request-presentation`
pub fn send_request_presentation(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
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

    let current_state: State = get_current_state(&thid, &UserType::Verifier)?.parse()?;

    match current_state {
        State::ReceiveProposePresentation | State::Unknown => {
            save_state(&thid, &State::SendPresentationRequest, &UserType::Verifier)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::SendPresentationRequest
            )))
        }
    };

    let request_message = get_presentation_exchange_message(
        PresentationExchangeType::RequestPresentation,
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
        &State::SendPresentationRequest,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/propose-presentation`
pub fn receive_propose_presentation(_options: &str, message: &str) -> StepResult {
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
    let base_info = get_from_to_from_message(&base_message)?;
    let presentation_exchange_data = exchange_info
        .presentation_exchange_data
        .ok_or("Presentation exchange data not provided.")?;

    let current_state: State = get_current_state(&thid, &UserType::Verifier)?.parse()?;

    match current_state {
        State::Unknown | State::SendPresentationRequest => save_state(
            &thid,
            &State::ReceiveProposePresentation,
            &UserType::Verifier,
        )?,
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::ReceiveProposePresentation
            )))
        }
    };

    save_presentation_exchange(
        &base_info.to,
        &base_info.from,
        &thid,
        &serde_json::to_string(&presentation_exchange_data)?,
        &State::ReceiveProposePresentation,
    )?;

    generate_step_output(message, "{}")
}

/// Protocol handler for direction: `receive`, type: `PRESENTATION_EXCHANGE_PROTOCOL_URI/presentation`
pub fn receive_presentation(_options: &str, message: &str) -> StepResult {
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
    let base_info = get_from_to_from_message(&base_message)?;
    let presentation_exchange_data = exchange_info
        .presentation_exchange_data
        .ok_or("Presentation exchange data not provided.")?;

    let req_data_saved = get_presentation_exchange(
        &base_info.to,
        &base_info.from,
        &thid,
        &State::SendPresentationRequest,
    )?;

    let result = validate_presentation_against_credentials(
        req_data_saved,
        presentation_exchange_data.clone(),
    );

    match result {
        Ok(_) => {}
        Err(err) => return Err(err),
    }

    let current_state: State = get_current_state(&thid, &UserType::Verifier)?.parse()?;

    match current_state {
        State::SendPresentationRequest => {
            save_state(&thid, &State::ReceivePresentation, &UserType::Verifier)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::ReceivePresentation
            )))
        }
    };

    save_presentation_exchange(
        &base_info.to,
        &base_info.from,
        &thid,
        &serde_json::to_string(&presentation_exchange_data)?,
        &State::ReceivePresentation,
    )?;

    generate_step_output(message, "{}")
}
