use k256::elliptic_curve::rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    datatypes::{BaseMessage, CommunicationDidDocument, MessageWithBody},
    get_from_to_from_message,
    keypair::save_com_keypair,
    protocols::protocol::{generate_step_output, generate_step_output_decrypted, StepResult},
};

use super::helper::{get_did_exchange_message, get_exchange_info_from_message, DIDExchangeType};

/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/request`
/// Uses the protocols/did_exchange/helper.rs/get_did_exchange_message to construct the request message,
/// that should be sent. Message will be sent NOT encrypted. (the other party does not have any keys
/// to decrypt the message)
/// Creates and stores a new communication keypair, that will be used for further communication with
/// the target DID.
pub fn send_request(message: &str) -> StepResult {
    let parsed_message: BaseMessage = serde_json::from_str(message)?;
    let exchange_info = get_from_to_from_message(parsed_message)?;
    let secret_key = StaticSecret::new(OsRng);
    let pub_key = PublicKey::from(&secret_key);
    let encoded_keypair = save_com_keypair(
        &exchange_info.from,
        &exchange_info.to,
        &hex::encode(pub_key.to_bytes()),
        &hex::encode(secret_key.to_bytes()),
        None,
        None,
    )?;
    let metadata = serde_json::to_string(&encoded_keypair)?;
    let request_message = get_did_exchange_message(
        DIDExchangeType::REQUEST,
        &exchange_info.from,
        &exchange_info.to,
        "",
        &encoded_keypair.pub_key,
    )?;

    return generate_step_output_decrypted(&serde_json::to_string(&request_message)?, &metadata);
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/request`
/// Receives the partners DID and communication pub key and generates new communication keypairs,
/// stores it within the rocks.db.
pub fn receive_request(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<CommunicationDidDocument> = serde_json::from_str(message)?;
    let exchange_info = get_exchange_info_from_message(parsed_message)?;
    let secret_key = StaticSecret::new(OsRng);
    let pub_key = PublicKey::from(&secret_key);

    let encoded_keypair = save_com_keypair(
        &exchange_info.to,
        &exchange_info.from,
        &hex::encode(pub_key.to_bytes()),
        &hex::encode(secret_key.to_bytes()),
        Some(String::from(exchange_info.pub_key_hex)),
        Some(String::from(exchange_info.service_endpoint)),
    )?;
    let metadata = serde_json::to_string(&encoded_keypair)?;

    return generate_step_output(message, &metadata);
}
