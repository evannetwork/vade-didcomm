use super::helper::{
    get_did_document_from_body,
    get_did_exchange_message,
    get_exchange_info_from_message,
    DidExchangeBaseMessage,
    DidExchangeOptions,
    DidExchangeType,
};
use crate::{
    get_from_to_from_message,
    keypair::{get_com_keypair, get_key_agreement_key, save_com_keypair},
    protocols::{
        did_exchange::datatypes::{State, UserType},
        did_exchange::did_exchange::{get_current_state, save_didexchange, save_state},
        protocol::{generate_step_output, StepResult}
    },
};

/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/response`
/// Uses the protocols/did_exchange/helper.rs/get_did_exchange_message to construct the request message,
/// that should be sent. Message will be sent NOT encrypted. (the other party does not have the
/// comm pub key to decrypt the message)
/// Constructs a message including the communication pub key, that was generated during receive_request.
pub fn send_response(options: &str, message: &str) -> StepResult {
    let parsed_message: DidExchangeBaseMessage = serde_json::from_str(message)?;
    let options: DidExchangeOptions = serde_json::from_str(options)?;
    let exchange_info = get_from_to_from_message(&parsed_message.base_message)?;
    let encoded_keypair = get_com_keypair(&exchange_info.from, &exchange_info.to)?;
    let metadata = serde_json::to_string(&encoded_keypair)?;
    let pub_key_bytes = hex::decode(encoded_keypair.pub_key)?;
    let pub_key_base58_string = &bs58::encode(pub_key_bytes).into_string();
    let (request_message, ..) = get_did_exchange_message(
        DidExchangeType::Response,
        &exchange_info.from,
        &encoded_keypair.key_agreement_key,
        &encoded_keypair.target_key_agreement_key,
        &options.service_endpoint.unwrap_or_else(|| "".to_string()),
        pub_key_base58_string,
        &parsed_message,
    )?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &UserType::Invitee)?.parse()?;
    match current_state {
        State::ReceiveRequest => {
            save_state(&thid, &State::SendResponse, &UserType::Invitee)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::SendResponse
            )))
        }
    };

    save_didexchange(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&request_message)?,
        &State::SendResponse,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, &metadata)
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/response`
/// Receives the partners pub key and updates the existing communication key pair for this DID in
/// the db.
pub fn receive_response(_options: &str, message: &str) -> StepResult {
    let parsed_message: DidExchangeBaseMessage = serde_json::from_str(message)?;
    let did_document = get_did_document_from_body(message)?;
    let exchange_info = get_exchange_info_from_message(&parsed_message.base_message, did_document)?;
    let encoded_keypair = get_key_agreement_key(&exchange_info.to)?;

    let enhanced_encoded_keypair = save_com_keypair(
        &exchange_info.to,
        &exchange_info.from,
        &exchange_info.to,
        &exchange_info.did_id,
        &encoded_keypair.pub_key,
        &encoded_keypair.secret_key,
        Some(exchange_info.pub_key_hex.to_owned()),
        Some(exchange_info.service_endpoint.to_owned()),
    )?;
    // in case we received a DID document from a known DID and we might be using this documents
    // DID for communication in future, store key for documents DID as well
    if exchange_info.from != exchange_info.did_id {
        save_com_keypair(
            &exchange_info.to,
            &exchange_info.from,
            &exchange_info.to,
            &exchange_info.did_id,
            &encoded_keypair.pub_key,
            &encoded_keypair.secret_key,
            Some(exchange_info.pub_key_hex),
            Some(exchange_info.service_endpoint),
        )?;
    }

    let metadata = serde_json::to_string(&enhanced_encoded_keypair)?;

    let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

    let current_state: State = get_current_state(&thid, &UserType::Inviter)?.parse()?;
    match current_state {
        State::SendRequest => {
            save_state(&thid, &State::ReceiveResponse, &UserType::Inviter)?
        }
        _ => {
            return Err(Box::from(format!(
                "Error while processing step: State from {} to {} not allowed",
                current_state,
                State::ReceiveResponse
            )))
        }
    };

    save_didexchange(
        &exchange_info.from,
        &exchange_info.to,
        &thid,
        &serde_json::to_string(&enhanced_encoded_keypair)?,
        &State::ReceiveResponse,
    )?;

    generate_step_output(message, &metadata)
}
