use std::collections::HashMap;

use super::helper::{
    get_issue_credential_info_from_message,
    get_issue_credential_message,
    IssueCredentialType,
};
use crate::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody},
    get_from_to_from_message,
    protocols::{
        issue_credential::{
            credential::{get_current_state, save_credential, save_state},
            datatypes::{CredentialData, State, UserType},
        },
        protocol::{generate_step_output, StepResult},
    },
};

/// Protocol handler for direction: `send`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/propose_credential`
pub fn send_propose_credential(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        body: HashMap::new(),
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(&base_message)?;

    let data =
        &serde_json::to_string(&parsed_message.body.ok_or("Credential data not provided.")?)?;
    let credential_data: CredentialData = serde_json::from_str(data)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty.")?;

    let current_state: State = get_current_state(&thid, &UserType::Holder)?.parse()?;

    match current_state {
        State::ReceiveOfferCredential | State::Unknown => {
            save_state(&thid, &State::SendProposeCredential, &UserType::Holder)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::SendProposeCredential
            )))
        }
    };

    let request_message = get_issue_credential_message(
        IssueCredentialType::ProposeCredential,
        &exchange_info.from,
        &exchange_info.to,
        credential_data.clone(),
        &thid,
    )?;

    save_credential(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&credential_data)?,
        &State::SendProposeCredential,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/offer_credential`
pub fn receive_offer_credential(_options: &str, message: &str) -> StepResult {
    let parsed_message: MessageWithBody<CredentialData> = serde_json::from_str(message)?;

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

    let exchange_info = get_issue_credential_info_from_message(parsed_message)?;
    let base_info = get_from_to_from_message(&base_message)?;
    let credential_data = exchange_info
        .credential_data
        .ok_or("Credential data not provided.")?;

    let current_state: State = get_current_state(&thid, &UserType::Holder)?.parse()?;

    match current_state {
        State::SendProposeCredential => {
            save_state(&thid, &State::ReceiveOfferCredential, &UserType::Holder)?
        }
        _ => {
            return Err(Box::from(format!(
                "State from {} to {} not allowed",
                current_state,
                State::ReceiveOfferCredential
            )))
        }
    };

    save_credential(
        &base_info.to,
        &base_info.from,
        &thid,
        &serde_json::to_string(&credential_data)?,
        &State::ReceiveOfferCredential,
    )?;

    generate_step_output(message, "{}")
}

/// Protocol handler for direction: `send`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/request_credential`
pub fn send_request_credential(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        body: HashMap::new(),
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(&base_message)?;

    let data =
        &serde_json::to_string(&parsed_message.body.ok_or("Credential data not provided.")?)?;
    let credential_data: CredentialData = serde_json::from_str(data)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &UserType::Holder)?.parse()?;

    match current_state {
        State::ReceiveOfferCredential | State::Unknown => {
            save_state(&thid, &State::SendRequestCredential, &UserType::Holder)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::SendRequestCredential
            )))
        }
    };

    let request_message = get_issue_credential_message(
        IssueCredentialType::RequestCredential,
        &exchange_info.from,
        &exchange_info.to,
        credential_data.clone(),
        &thid,
    )?;

    save_credential(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&credential_data)?,
        &State::SendRequestCredential,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/issue_credential`
pub fn receive_issue_credential(_options: &str, message: &str) -> StepResult {
    let parsed_message: MessageWithBody<CredentialData> = serde_json::from_str(message)?;

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

    let exchange_info = get_issue_credential_info_from_message(parsed_message)?;
    let base_info = get_from_to_from_message(&base_message)?;
    let credential_data = exchange_info
        .credential_data
        .ok_or("Credential data not provided.")?;

    let current_state: State = get_current_state(&thid, &UserType::Holder)?.parse()?;

    match current_state {
        State::SendRequestCredential => {
            save_state(&thid, &State::ReceiveIssueCredential, &UserType::Holder)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::ReceiveIssueCredential
            )))
        }
    };

    save_credential(
        &base_info.to,
        &base_info.from,
        &thid,
        &serde_json::to_string(&credential_data)?,
        &State::ReceiveIssueCredential,
    )?;

    generate_step_output(message, "{}")
}
