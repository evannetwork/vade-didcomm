use k256::elliptic_curve::rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    get_step_output, get_step_output_decrypted, save_com_keypair, BaseMessage, MessageWithBody,
    StepResult,
};

use super::helper::{get_did_exchange_message, DidcommObj};

/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/request`
/// Uses the protocols/did_exchange/helper.rs/get_did_exchange_message to construct the request message,
/// that should be sent. Message will be sent NOT encrypted. (the other party does not have any keys
/// to decrypt the message)
/// Creates and stores a new communication keypair, that will be used for further communication with
/// the target did.
pub fn send_request(message: &str) -> StepResult {
    let parsed_message: BaseMessage = serde_json::from_str(message)?;
    let from_did = parsed_message.from.as_ref().ok_or("from is required")?;
    let to_vec = parsed_message.to.as_ref().ok_or("to is required")?;
    let to_did = &to_vec[0];
    let secret_key = StaticSecret::new(OsRng);
    let pub_key = PublicKey::from(&secret_key);
    let encoded_keypair = save_com_keypair(
        from_did,
        to_did,
        &hex::encode(pub_key.to_bytes()),
        &hex::encode(secret_key.to_bytes()),
        None,
        None,
    )?;
    let metadata = serde_json::to_string(&encoded_keypair)?;
    let request_message =
        get_did_exchange_message("request", &from_did, to_did, "", &encoded_keypair)?;

    return get_step_output_decrypted(&serde_json::to_string(&request_message)?, &metadata);
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/request`
/// Receives the partners did and communication pub key and generates new communication keypairs,
/// stores it within the rocks.db.
pub fn receive_request(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<DidcommObj> = serde_json::from_str(message)?;
    let from_did = parsed_message.from.as_ref().ok_or("from is required")?;
    let to_vec = parsed_message.to.as_ref().ok_or("to is required")?;
    let to_did = &to_vec[0];
    let didcomm_obj: DidcommObj = parsed_message.body.ok_or("body is required")?;
    let pub_key_hex = &didcomm_obj.public_key[0].public_key_base_58;
    let service_endpoint = &didcomm_obj.service[0].service_endpoint;

    let secret_key = StaticSecret::new(OsRng);
    let pub_key = PublicKey::from(&secret_key);

    let encoded_keypair = save_com_keypair(
        to_did,
        from_did,
        &hex::encode(pub_key.to_bytes()),
        &hex::encode(secret_key.to_bytes()),
        Some(String::from(pub_key_hex)),
        Some(String::from(service_endpoint)),
    )?;
    let metadata = serde_json::to_string(&encoded_keypair)?;

    return get_step_output(message, &metadata);
}
