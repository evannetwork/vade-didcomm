use crate::{
    datatypes::{PresentProofReq, MessageWithBody},
    get_from_to_from_message,
    keypair::save_com_keypair,
    protocols::protocol::{generate_step_output, StepResult},
};

use super::helper::{get_present_proof_message, get_present_proof_info_from_message, PresentProofType};

/// protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/request`
/// Uses the protocols/present_proof/helper.rs/get_did_exchange_message to construct the request message,
pub fn send_request_presentation(message: &str) -> StepResult {
    let parsed_message: PresentProofReq = serde_json::from_str(message)?;
    let exchange_info = get_from_to_from_message(parsed_message.baseMessage)?; 

    let request_message = get_present_proof_message(
        PresentProofType::REQUEST_PRESENTATION,
        &exchange_info.from,
        &exchange_info.to,
        &parsed_message.servicePoint,
        &parsed_message.requestPresentation,
    )?;
 
    return generate_step_output(&serde_json::to_string(&request_message)?, "{}");
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/request`
/// Receives the partners DID and communication pub key and generates new communication keypairs,
/// stores it within the db.
pub fn receive_presentation(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<String> = serde_json::from_str(message)?;
    let exchange_info = get_present_proof_info_from_message(parsed_message)?;


    // let encoded_keypair = save_com_keypair(
    //     &exchange_info.to,
    //     &exchange_info.from,
    //     &hex::encode(pub_key.to_bytes()),
    //     &hex::encode(secret_key.to_bytes()),
    //     Some(String::from(exchange_info.pub_key_hex)),
    //     Some(String::from(exchange_info.service_endpoint)),
    // )?;

    return generate_step_output(message, "{}");
}


/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/request`
/// Uses the protocols/did_exchange/helper.rs/get_did_exchange_message to construct the request message,
/// that should be sent. Message will be sent NOT encrypted. (the other party does not have any keys
/// to decrypt the message)
/// Creates and stores a new communication keypair, that will be used for further communication with
/// the target DID.
pub fn receive_propose_presentation(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<String> = serde_json::from_str(message)?;
    let exchange_info = get_present_proof_info_from_message(parsed_message)?;


    // let encoded_keypair = save_com_keypair(
    //     &exchange_info.to,
    //     &exchange_info.from,
    //     &hex::encode(pub_key.to_bytes()),
    //     &hex::encode(secret_key.to_bytes()),
    //     Some(String::from(exchange_info.pub_key_hex)),
    //     Some(String::from(exchange_info.service_endpoint)),
    // )?;

    return generate_step_output(message, "{}");
}