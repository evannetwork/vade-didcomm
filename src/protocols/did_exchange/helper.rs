use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    datatypes::{
        CommunicationDidDocument, DIDCommPubKey, DIDCommService, ExchangeInfo, MessageWithBody,
        DID_EXCHANGE_PROTOCOL_URL,
    },
    utils::SyncResult,
};

/// Specifies all possible message directions.
#[derive(PartialEq)]
pub enum DIDExchangeType {
    REQUEST,
    RESPONSE,
}

/// Creates a new communication DIDComm object for a specific DID, a communication pub key and the
/// service url, where the user can be reached.
///
/// # Arguments
/// * `from_did` - DID to build the DIDComm obj for
/// * `public_key_encoded` - communication pub key for the DID exchange that will be sent to the target
/// * `service_endpoint` - url where the user can be reached
///
/// # Returns
/// * `CommunicationDidDocument` - constructed DIDComm object, ready to be sent
pub fn get_communication_did_doc(
    from_did: &str,
    public_key_encoded: &str,
    service_endpoint: &str,
) -> CommunicationDidDocument {
    let mut pub_key_vec = Vec::new();
    pub_key_vec.push(DIDCommPubKey {
        id: format!("{}#key-1", from_did),
        r#type: [String::from("Ed25519VerificationKey2018")].to_vec(),
        public_key_base_58: format!("{}", public_key_encoded),
    });

    let mut service_vec = Vec::new();
    service_vec.push(DIDCommService {
        id: format!("{}#didcomm", from_did),
        r#type: String::from("did-communication"),
        priority: 0,
        service_endpoint: format!("{}", service_endpoint),
        recipient_keys: [format!("{}", public_key_encoded)].to_vec(),
    });

    return CommunicationDidDocument {
        context: String::from("https://w3id.org/did/v1"),
        id: format!("{}", from_did),
        public_key: pub_key_vec,
        authentication: [String::from("{0}#key-1")].to_vec(),
        service: service_vec,
    };
}

/// Constructs a new DID exchange message, including the DIDComm object as message body.
///
/// # Arguments
/// * `step_type` - step to build the message type (request, response)
/// * `from_did` - DID that sends the message
/// * `to_did` - DID that receives the message
/// * `from_service_endpoint` - url where the user can be reached
/// * `encoded_keypair` - communication keypair (only pubkey will be used)
///
/// # Returns
/// * `MessageWithBody<CommunicationDidDocument>` - constructed DIDComm object, ready to be sent
pub fn get_did_exchange_message(
    step_type: DIDExchangeType,
    from_did: &str,
    to_did: &str,
    from_service_endpoint: &str,
    pub_key: &str,
) -> SyncResult<MessageWithBody<CommunicationDidDocument>> {
    let did_comm_obj = get_communication_did_doc(from_did, pub_key, from_service_endpoint);
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let service_id = format!("{0}#key-1", from_did);
    let step_name = match step_type {
        DIDExchangeType::REQUEST => "request",
        DIDExchangeType::RESPONSE => "response",
    };
    let exchange_request: MessageWithBody<CommunicationDidDocument> = MessageWithBody {
        id: Some(String::from(&thread_id)),
        pthid: Some(format!("{}#key-1", String::from(thread_id))),
        thid: Some(service_id),
        from: Some(String::from(from_did)),
        to: Some([String::from(to_did)].to_vec()),
        body: Some(did_comm_obj),
        r#type: format!("{}/{}", DID_EXCHANGE_PROTOCOL_URL, step_name),
        other: HashMap::new(),
    };

    return Ok(exchange_request);
}

/// Takes an DIDComm message and extracts all necessary information to process it during request /
/// response.
///
/// # Arguments
/// * `message` - DIDComm message with communication DID document as body
///
/// # Returns
/// * `ExchangeInfo` - necessary information
pub fn get_exchange_info_from_message(
    message: MessageWithBody<CommunicationDidDocument>,
) -> SyncResult<ExchangeInfo> {
    let from_did = message.from.ok_or("from is required")?;

    let to_vec = message.to.ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from(
            "DID exchange requires at least one DID in the to field.",
        ));
    }
    let to_did = &to_vec[0];
    let didcomm_obj: CommunicationDidDocument = message.body.ok_or("body is required")?;
    if didcomm_obj.public_key.is_empty() {
        return Err(Box::from(
            "No pub key was attached to the communication DID document.",
        ));
    }
    let pub_key_hex = &didcomm_obj.public_key[0].public_key_base_58;
    if didcomm_obj.service.is_empty() {
        return Err(Box::from(
            "No service_endpoint was attached to the communication DID document.",
        ));
    }
    let service_endpoint = &didcomm_obj.service[0].service_endpoint;

    return Ok(ExchangeInfo {
        from: String::from(from_did),
        to: String::from(to_did),
        pub_key_hex: String::from(pub_key_hex),
        service_endpoint: String::from(service_endpoint),
    });
}