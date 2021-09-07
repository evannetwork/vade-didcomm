use crate::{
    datatypes::{BaseMessage, ExtendedMessage, MessageWithBody},
    get_from_to_from_message,
    protocols::issue_credential::credential::{get_current_state, save_credential, save_state},
    protocols::issue_credential::datatypes::{CredentialData, State, UserType},
    protocols::protocol::{generate_step_output, StepResult},
};

use super::helper::{
    get_issue_credential_info_from_message,
    get_issue_credential_message,
    IssueCredentialType,
};

/// Protocol handler for direction: `send`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/offer_credential`
pub fn send_offer_credential(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(base_message)?;

    let data =
        &serde_json::to_string(&parsed_message.body.ok_or("Credential data not provided.")?)?;
    let credential_data: CredentialData = serde_json::from_str(&data)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let request_message = get_issue_credential_message(
        IssueCredentialType::OfferCredential,
        &exchange_info.from,
        &exchange_info.to,
        credential_data.clone(),
        &thid,
    )?;

    let current_state: State = get_current_state(&thid, &UserType::Issuer)?.parse()?;
    match current_state {
        State::ReceiveProposeCredential | State::Unknown => {
            save_state(&thid, &State::SendOfferCredential, &UserType::Issuer)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::SendOfferCredential
            )))
        }
    };

    save_credential(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&credential_data)?,
        &State::SendOfferCredential,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/request_credential`
pub fn receive_request_credential(_options: &str, message: &str) -> StepResult {
    let parsed_message: MessageWithBody<CredentialData> = serde_json::from_str(message)?;
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
    let base_info = get_from_to_from_message(base_message)?;

    let current_state: State = get_current_state(&thid, &UserType::Issuer)?.parse()?;

    match current_state {
        State::SendOfferCredential => {
            save_state(&thid, &State::ReceiveRequestCredential, &UserType::Issuer)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::ReceiveRequestCredential,
            )))
        }
    };

    let exchange_info = get_issue_credential_info_from_message(parsed_message)?;

    let credential_data = exchange_info
        .credential_data
        .ok_or("Credential data not provided.")?;

    save_credential(
        &base_info.from,
        &base_info.to,
        &thid,
        &serde_json::to_string(&credential_data)?,
        &State::ReceiveRequestCredential,
    )?;

    generate_step_output(message, "{}")
}

/// Protocol handler for direction: `receive`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/propose-credential`
pub fn receive_propose_credential(_options: &str, message: &str) -> StepResult {
    let parsed_message: MessageWithBody<CredentialData> = serde_json::from_str(message)?;
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

    let base_info = get_from_to_from_message(base_message)?;
    let thid = parsed_message
        .thid
        .to_owned()
        .ok_or("Thread id can't be empty")?;

    let exchange_info = get_issue_credential_info_from_message(parsed_message)?;

    let credential_data = exchange_info
        .credential_data
        .ok_or("Credential data not provided.")?;

    let current_state: State = get_current_state(&thid, &UserType::Issuer)?.parse()?;
    match current_state {
        State::SendOfferCredential | State::Unknown => {
            save_state(&thid, &State::ReceiveProposeCredential, &UserType::Issuer)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::ReceiveProposeCredential
            )))
        }
    };

    save_credential(
        &base_info.from,
        &base_info.to,
        &thid,
        &serde_json::to_string(&credential_data)?,
        &State::ReceiveProposeCredential,
    )?;

    generate_step_output(message, "{}")
}

/// Protocol handler for direction: `send`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/issue_credential`
pub fn send_issue_credential(_options: &str, message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let base_message: BaseMessage = BaseMessage {
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(base_message)?;

    let data =
        &serde_json::to_string(&parsed_message.body.ok_or("Credential data not provided.")?)?;
    let credential_data: CredentialData = serde_json::from_str(&data)?;
    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let request_message = get_issue_credential_message(
        IssueCredentialType::IssueCredential,
        &exchange_info.from,
        &exchange_info.to,
        credential_data.clone(),
        &thid,
    )?;

    let current_state: State = get_current_state(&thid, &UserType::Issuer)?.parse()?;
    match current_state {
        State::ReceiveRequestCredential => {
            save_state(&thid, &State::SendIssueCredential, &UserType::Issuer)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::SendIssueCredential
            )))
        }
    };

    save_credential(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&credential_data)?,
        &State::SendIssueCredential,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}
