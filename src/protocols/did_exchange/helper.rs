use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    datatypes::{
        CommKeyPair, DidCommPubKey, DidCommService, DidcommObj, MessageWithBody,
        DID_EXCHANGE_PROTOCOL_URL,
    },
    utils::SyncResult,
};

/// Creates a new communication didcomm object for a specific did, a communication pub key and the
/// service url, where the user can be reached.
///
/// # Arguments
/// * `from_did` - did to build the didcomm obj for
/// * `public_key_encoded` - communication pub key for the did exchange that will be sent to the target
/// * `service_endpoint` - url where the user can be reached
///
/// # Returns
/// * `DidcommObj` - constructed didcomm object, ready to be sent
pub fn get_com_did_obj(
    from_did: &str,
    public_key_encoded: &str,
    service_endpoint: &str,
) -> DidcommObj {
    let mut pub_key_vec = Vec::new();
    pub_key_vec.push(DidCommPubKey {
        id: format!("{}#key-1", from_did),
        r#type: [String::from("Ed25519VerificationKey2018")].to_vec(),
        public_key_base_58: format!("{}", public_key_encoded),
    });

    let mut service_vec = Vec::new();
    service_vec.push(DidCommService {
        id: format!("{}#didcomm", from_did),
        r#type: String::from("did-communication"),
        priority: 0,
        service_endpoint: format!("{}", service_endpoint),
        recipient_keys: [format!("{}", public_key_encoded)].to_vec(),
    });

    return DidcommObj {
        context: String::from("https://w3id.org/did/v1"),
        id: format!("{}", from_did),
        public_key: pub_key_vec,
        authentication: [String::from("{0}#key-1")].to_vec(),
        service: service_vec,
    };
}

/// Constructs a new did exchange message, including the didcomm object as message body.
///
/// # Arguments
/// * `step_type` - step to build the message type (request, response)
/// * `from_did` - did that sends the message
/// * `to_did` - did that receives the message
/// * `from_service_endpoint` - url where the user can be reached
/// * `encoded_keypair` - communication keypair (only pubkey will be used)
///
/// # Returns
/// * `MessageWithBody<DidcommObj>` - constructed didcomm object, ready to be sent
pub fn get_did_exchange_message(
    step_type: &str,
    from_did: &str,
    to_did: &str,
    from_service_endpoint: &str,
    encoded_keypair: &CommKeyPair,
) -> SyncResult<MessageWithBody<DidcommObj>> {
    let did_comm_obj = get_com_did_obj(from_did, &encoded_keypair.pub_key, from_service_endpoint);
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let service_id = format!("{0}#key-1", from_did);
    let exchange_request: MessageWithBody<DidcommObj> = MessageWithBody {
        id: Some(String::from(&thread_id)),
        pthid: Some(format!("{}#key-1", String::from(thread_id))),
        thid: Some(service_id),
        from: Some(String::from(from_did)),
        to: Some([String::from(to_did)].to_vec()),
        body: Some(did_comm_obj),
        r#type: format!("{}/{}", DID_EXCHANGE_PROTOCOL_URL, step_type),
        other: HashMap::new(),
    };

    return Ok(exchange_request);
}
