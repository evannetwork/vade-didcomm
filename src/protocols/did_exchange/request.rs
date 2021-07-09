use std::collections::HashMap;

use uuid::Uuid;
use k256::elliptic_curve::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{BaseMessage, MessageWithBody, StepResult, get_step_output, get_step_output_decrypted, protocol::DID_EXCHANGE_PROTOCOL_URL, utils::SyncResult, vec_to_array, write_db};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommKeyPair {
    pub pub_key: String,
    pub secret_key: String,
    pub shared_secret: String,
    pub target_pub_key: String,
    pub target_service_endpoint: String,
}

pub fn pub_key_to_string(pub_key: ed25519_dalek::PublicKey) -> String {
    return hex::encode(pub_key.to_bytes());
}

pub fn save_com_keypair(
    from_did: &str,
    to_did: &str,
    pub_key: &str,
    secret_key: &str,
    target_pub_key: Option<String>,
    service_endpoint: Option<String>,
    shared_secret: Option<String>,
) -> SyncResult<CommKeyPair> {
    let comm_keypair = CommKeyPair {
        pub_key: String::from(pub_key),
        secret_key: String::from(secret_key),
        target_pub_key: target_pub_key.unwrap_or(String::from("")),
        target_service_endpoint: service_endpoint.unwrap_or(String::from("")),
        shared_secret: shared_secret.unwrap_or(String::from("")),
    };

    let _ = write_db(
        &format!("comm_keypair_{}_{}", from_did, to_did),
        &serde_json::to_string(&comm_keypair)?,
    );

    return Ok(comm_keypair);
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

pub fn get_request_message(
    from_did: &str,
    to_did: &str,
    from_service_endpoint: &str,
    encoded_keypair: CommKeyPair,
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
        r#type: format!("{}/request", DID_EXCHANGE_PROTOCOL_URL),
        other: HashMap::new(),
    };

    return Ok(exchange_request);
}

pub fn send_request(message: &str) -> StepResult {
    let parsed_message: BaseMessage = serde_json::from_str(message)?;
    let from_did = parsed_message.from.as_ref().ok_or("from is required")?;
    let to_vec = parsed_message.to.as_ref().ok_or("to is required")?;
    let to_did = &to_vec[0];
    let keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
    let secret_key = StaticSecret::new(OsRng);
    let pub_key = PublicKey::from(&secret_key);
    let encoded_keypair = save_com_keypair(
        from_did,
        to_did,
        &hex::encode(secret_key.to_bytes()),
        &hex::encode(pub_key.to_bytes()),
        None,
        None,
        None,
    )?;
    let metadata = serde_json::to_string(&encoded_keypair)?;
    let request_message = get_request_message(&from_did, to_did, "", encoded_keypair)?;

    return get_step_output_decrypted(
        &serde_json::to_string(&request_message)?,
        &metadata,
    );
}

pub fn receive_request(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<DidcommObj> = serde_json::from_str(message)?;
    let from_did = parsed_message.from.as_ref().ok_or("from is required")?;
    let to_vec = parsed_message.to.as_ref().ok_or("to is required")?;
    let to_did = &to_vec[0];
    let thread_id = parsed_message.thid.ok_or("thid is required")?;
    let didcomm_obj: DidcommObj = parsed_message.body.ok_or("body is required")?;
    let pub_key_hex = &didcomm_obj.public_key[0].public_key_base_58;
    let service_endpoint = &didcomm_obj.service[0].service_endpoint;
    let keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);

    let secret_key = StaticSecret::new(OsRng);
    let pub_key = PublicKey::from(&secret_key);
    let decoded_target_pub_key = vec_to_array(hex::decode(pub_key_hex)?);
    let target_pub_key = PublicKey::from(decoded_target_pub_key);
    let shared_secret = secret_key.diffie_hellman(&target_pub_key);

    let encoded_keypair = save_com_keypair(
        to_did,
        from_did,
        &hex::encode(secret_key.to_bytes()),
        &hex::encode(pub_key.to_bytes()),
        Some(String::from(pub_key_hex)),
        Some(String::from(service_endpoint)),
        Some(hex::encode(shared_secret.to_bytes())),
    )?;

    let metadata = serde_json::to_string(&encoded_keypair)?;

    return get_step_output(message, &metadata);
}
