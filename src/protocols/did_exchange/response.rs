#[cfg(not(feature = "state_storage"))]
use x25519_dalek::{PublicKey, StaticSecret};

use super::helper::{
    get_did_document_from_body,
    get_did_exchange_message,
    get_exchange_info_from_message,
    DidExchangeBaseMessage,
    DidExchangeOptions,
    DidExchangeType,
};
#[cfg(not(feature = "state_storage"))]
use crate::datatypes::CommKeyPair;
use crate::{
    get_from_to_from_message,
    protocols::protocol::{generate_step_output, StepResult},
};
#[cfg(feature = "state_storage")]
use crate::{
    keypair::{get_com_keypair, get_key_agreement_key, save_com_keypair},
    protocols::did_exchange::{
        datatypes::{State, UserType},
        did_exchange::{get_current_state, save_didexchange, save_state},
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

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let encoded_keypair = get_com_keypair(&exchange_info.from, &exchange_info.to)?;
            pub_key_bytes = hex::decode(encoded_keypair.pub_key)?;
        } else {
            let secret_key = options
                .did_exchange_my_secret
                .map(StaticSecret::from)
                .ok_or("did_exchange_my_secret is required when sending response without storage")?;
            let pub_key = PublicKey::from(&secret_key);

            let pub_key_bytes = pub_key.to_bytes();
        }
    }

    let codec: &[u8] = &[0xec, 0x1];
    let data = [codec, &pub_key_bytes].concat();
    let key_agreement_key = format!("did:key:z{}", bs58::encode(data).into_string());

    let pub_key_base58_string = &bs58::encode(pub_key_bytes).into_string();
    let (request_message, ..) = get_did_exchange_message(
        DidExchangeType::Response,
        &exchange_info.from,
        &key_agreement_key,
        &exchange_info.to,
        &options.service_endpoint.unwrap_or_default(),
        pub_key_base58_string,
        &parsed_message,
    )?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
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
    } else { }
    }

    generate_step_output(&serde_json::to_string(&request_message)?, "{}")
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/response`
/// Receives the partners pub key and updates the existing communication key pair for this DID in
/// the db.
pub fn receive_response(
    #[allow(unused_variables)] // may not be used, depending on feature setup
    options: &str,
    message: &str,
) -> StepResult {
    let parsed_message: DidExchangeBaseMessage = serde_json::from_str(message)?;
    let did_document = get_did_document_from_body(message)?;
    let exchange_info = get_exchange_info_from_message(&parsed_message.base_message, did_document)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
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
            comm_key_pair = &enhanced_encoded_keypair;
        } else {
            let options: DidExchangeOptions = serde_json::from_str(options)?;
            let secret_key = options
                .did_exchange_my_secret
                .map(StaticSecret::from)
                .ok_or("did_exchange_my_secret is required when receiving response without storage")?;
            let pub_key = PublicKey::from(&secret_key);
            let comm_key_pair = CommKeyPair {
                pub_key:  hex::encode(pub_key.to_bytes()),
                secret_key:  hex::encode(secret_key.to_bytes()),
                key_agreement_key: exchange_info.to,
                target_key_agreement_key: exchange_info.did_id,
                target_pub_key: exchange_info.pub_key_hex,
                target_service_endpoint: exchange_info.service_endpoint,
            };
        }
    }

    let metadata = serde_json::to_string(&comm_key_pair)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
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
        } else { }
    }

    generate_step_output(message, &metadata)
}
