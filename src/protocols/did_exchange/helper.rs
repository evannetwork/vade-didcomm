use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use uuid::Uuid;

use crate::{CommKeyPair, MessageWithBody, protocol::DID_EXCHANGE_PROTOCOL_URL, utils::SyncResult};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCommPubKey {
    pub id: String,
    pub public_key_base_58: String,
    pub r#type: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCommService {
    pub id: String,
    pub r#type: String,
    pub priority: u8,
    pub service_endpoint: String,
    pub recipient_keys: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommObj {
    #[serde(rename(serialize = "@context", deserialize = "@context"))]
    pub context: String,
    pub id: String,
    pub authentication: Vec<String>,
    pub public_key: Vec<DidCommPubKey>,
    pub service: Vec<DidCommService>,
}

pub fn get_com_did_obj(
    from_did: &str,
    public_key_encoded: &str,
    service_endpoint: &str,
) -> DidcommObj {
    let mut pub_key_vec = Vec::new();
    pub_key_vec.push(DidCommPubKey {
        id: format!("{}#key-1", from_did),
        r#type: [
            String::from("Ed25519VerificationKey2018"),
        ].to_vec(),
        public_key_base_58: format!("{}", public_key_encoded),
    });

    let mut service_vec = Vec::new();
    service_vec.push(DidCommService {
        id: format!("{}#didcomm", from_did),
        r#type: String::from("did-communication"),
        priority: 0,
        service_endpoint: format!("{}", service_endpoint),
        recipient_keys: [
            format!("{}", public_key_encoded),
        ].to_vec(),
    });

    return DidcommObj {
        context: String::from("https://w3id.org/did/v1"),
        id: format!("{}", from_did),
        public_key: pub_key_vec,
        authentication: [
            String::from("{0}#key-1"),
        ].to_vec(),
        service: service_vec,
    }
}

pub fn get_did_exchange_message(
    step_type: &str,
    from_did: &str,
    to_did: &str,
    from_service_endpoint: &str,
    encoded_keypair: &CommKeyPair,
) -> SyncResult<MessageWithBody<DidcommObj>> {
    let did_comm_obj = get_com_did_obj(
        from_did,
        &encoded_keypair.pub_key,
        from_service_endpoint,
    );
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
