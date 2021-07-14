use crate::{
    get_com_keypair, get_step_output, get_step_output_decrypted, save_com_keypair, BaseMessage,
    MessageWithBody, StepResult,
};

use super::helper::{get_did_exchange_message, DidcommObj};

/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/response`
/// Uses the protocols/did_exchange/helper.rs/get_did_exchange_message to construct the request message,
/// that should be sent. Message will be sent NOT encrypted. (the other party does not have the
/// comm pub key to decrypt the message)
/// Constructs a message including the communication pub key, that was generated during receive_request.
pub fn send_response(message: &str) -> StepResult {
    let parsed_message: BaseMessage = serde_json::from_str(message)?;
    let from_did = parsed_message.from.as_ref().ok_or("from is required")?;
    let to_vec = parsed_message.to.as_ref().ok_or("to is required")?;
    let to_did = &to_vec[0];
    let encoded_keypair = get_com_keypair(from_did, to_did)?;
    let metadata = serde_json::to_string(&encoded_keypair)?;
    let request_message =
        get_did_exchange_message("response", &from_did, to_did, "", &encoded_keypair)?;

    return get_step_output_decrypted(&serde_json::to_string(&request_message)?, &metadata);
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/response`
/// Receives the partners pub key and updates the existing communication key pair for this did in
/// the rocks.db.
pub fn receive_response(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<DidcommObj> = serde_json::from_str(message)?;
    let from_did = parsed_message.from.as_ref().ok_or("from is required")?;
    let to_vec = parsed_message.to.as_ref().ok_or("to is required")?;
    let to_did = &to_vec[0];
    let didcomm_obj: DidcommObj = parsed_message.body.ok_or("body is required")?;
    let pub_key_hex = &didcomm_obj.public_key[0].public_key_base_58;
    let service_endpoint = &didcomm_obj.service[0].service_endpoint;

    let encoded_keypair = get_com_keypair(to_did, from_did)?;

    let encoded_keypair = save_com_keypair(
        to_did,
        from_did,
        &encoded_keypair.pub_key,
        &encoded_keypair.secret_key,
        Some(String::from(pub_key_hex)),
        Some(String::from(service_endpoint)),
    )?;

    let metadata = serde_json::to_string(&encoded_keypair)?;

    return get_step_output(message, &metadata);
}
