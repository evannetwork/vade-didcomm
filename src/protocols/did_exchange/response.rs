use crate::{
    datatypes::{BaseMessage, CommunicationDidDocument, MessageWithBody},
    get_from_to_from_message,
    keypair::{get_com_keypair, save_com_keypair},
    protocols::protocol::{generate_step_output, StepResult},
};

use super::helper::{get_did_exchange_message, get_exchange_info_from_message, DIDExchangeType};

/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/response`
/// Uses the protocols/did_exchange/helper.rs/get_did_exchange_message to construct the request message,
/// that should be sent. Message will be sent NOT encrypted. (the other party does not have the
/// comm pub key to decrypt the message)
/// Constructs a message including the communication pub key, that was generated during receive_request.
pub fn send_response(message: &str) -> StepResult {
    let parsed_message: BaseMessage = serde_json::from_str(message)?;
    let exchange_info = get_from_to_from_message(parsed_message)?;
    let encoded_keypair = get_com_keypair(&exchange_info.from, &exchange_info.to)?;
    let metadata = serde_json::to_string(&encoded_keypair)?;
    let request_message = get_did_exchange_message(
        DIDExchangeType::RESPONSE,
        &exchange_info.from,
        &exchange_info.to,
        "",
        &encoded_keypair.pub_key,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, &metadata)
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/response`
/// Receives the partners pub key and updates the existing communication key pair for this DID in
/// the db.
pub fn receive_response(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<CommunicationDidDocument> = serde_json::from_str(message)?;
    let exchange_info = get_exchange_info_from_message(parsed_message)?;
    let encoded_keypair = get_com_keypair(&exchange_info.to, &exchange_info.from)?;

    let enhanced_encoded_keypair = save_com_keypair(
        &exchange_info.to,
        &exchange_info.from,
        &encoded_keypair.pub_key,
        &encoded_keypair.secret_key,
        Some(exchange_info.pub_key_hex),
        Some(exchange_info.service_endpoint),
    )?;

    let metadata = serde_json::to_string(&enhanced_encoded_keypair)?;

    generate_step_output(message, &metadata)
}
