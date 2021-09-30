use k256::elliptic_curve::rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

use super::helper::{
    get_did_document_from_body,
    get_did_exchange_message,
    get_exchange_info_from_message,
    DidExchangeOptions,
    DidExchangeType,
};
use crate::{
    datatypes::BaseMessage,
    get_from_to_from_message,
    keypair::save_com_keypair,
    protocols::protocol::{generate_step_output, StepResult},
};

/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/request`
/// Uses the protocols/did_exchange/helper.rs/get_did_exchange_message to construct the request message,
/// that should be sent. Message will be sent NOT encrypted. (the other party does not have any keys
/// to decrypt the message)
/// Creates and stores a new communication keypair, that will be used for further communication with
/// the target DID.
pub fn send_request(options: &str, message: &str) -> StepResult {
    let parsed_message: BaseMessage = serde_json::from_str(message)?;
    let options: DidExchangeOptions = serde_json::from_str(options)?;
    let exchange_info = get_from_to_from_message(parsed_message)?;
    let secret_key = StaticSecret::new(OsRng);
    let pub_key = PublicKey::from(&secret_key);

    let codec: &[u8] = &[0xec, 0x1];
    let data = [codec, pub_key.as_bytes()].concat();
    let key_did = format!("did:key:z{}", bs58::encode(data).into_string());
    let encoded_keypair = save_com_keypair(
        &exchange_info.from,
        &exchange_info.to,
        &key_did,
        "",
        &hex::encode(pub_key.to_bytes()),
        &hex::encode(secret_key.to_bytes()),
        None,
        None,
    )?;
    let metadata = serde_json::to_string(&encoded_keypair)?;
    let pub_key_bytes = hex::decode(encoded_keypair.pub_key)?;
    let pub_key_base58_string = &bs58::encode(pub_key_bytes).into_string();
    let request_message = get_did_exchange_message(
        DidExchangeType::Request,
        &exchange_info.from,
        &key_did,
        &exchange_info.to,
        &options.service_endpoint.unwrap_or_else(|| "".to_string()),
        pub_key_base58_string,
    )?;

    generate_step_output(&serde_json::to_string(&request_message)?, &metadata)
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/request`
/// Receives the partners DID and communication pub key and generates new communication keypairs,
/// stores it within the db.
pub fn receive_request(_options: &str, message: &str) -> StepResult {
    let did_document = get_did_document_from_body(message)?;
    let parsed_message: BaseMessage = serde_json::from_str(message)?;
    let exchange_info = get_exchange_info_from_message(parsed_message, did_document)?;
    let secret_key = StaticSecret::new(OsRng);
    let pub_key = PublicKey::from(&secret_key);

    let codec: &[u8] = &[0xec, 0x1];
    let data = [codec, pub_key.as_bytes()].concat();
    let key_did = format!("did:key:z{}", bs58::encode(data).into_string());
    let encoded_keypair = save_com_keypair(
        &exchange_info.to,
        &exchange_info.from,
        &key_did,
        &exchange_info.did_id,
        &hex::encode(pub_key.to_bytes()),
        &hex::encode(secret_key.to_bytes()),
        Some(exchange_info.pub_key_hex),
        Some(exchange_info.service_endpoint),
    )?;
    let metadata = serde_json::to_string(&encoded_keypair)?;

    generate_step_output(message, &metadata)
}
