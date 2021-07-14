use k256::elliptic_curve::rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{
    get_step_output, get_step_output_decrypted,
    helper::{get_did_exchange_message, DidcommObj},
    save_com_keypair, BaseMessage, MessageWithBody, StepResult,
};

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
