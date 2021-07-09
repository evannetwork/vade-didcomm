use std::collections::HashMap;

use uuid::Uuid;
use k256::elliptic_curve::rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{BaseMessage, MessageWithBody, StepOutput, StepResult, get_step_output_decrypted, protocol::DID_EXCHANGE_PROTOCOL_URL, utils::SyncResult, write_db};

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
    pub recipientKeys: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommObj {
    #[serde(rename(serialize = "@context"))]
    pub context: String,
    pub id: String,
    pub authentication: Vec<String>,
    pub public_key: Vec<DidCommPubKey>,
    pub service: Vec<DidCommService>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommKeyPair {
    pub encoded_pub_key: String,
    pub encoded_secret_key: String,
    pub encoded_target_pub_key: String,
}

pub fn save_com_keypair(
    from_did: &str,
    to_did: &str,
    secret_key: &ed25519_dalek::SecretKey,
    public_key: &ed25519_dalek::PublicKey,
    target_pub_key: &Option<ed25519_dalek::PublicKey>,
) -> SyncResult<CommKeyPair> {
    let mut encoded_target_pub_key = String::from("");

    match target_pub_key {
        Some(value) => encoded_target_pub_key = hex::encode(value.to_bytes()),
        None => (),
    }

    let comm_keypair = CommKeyPair {
        encoded_pub_key: hex::encode(public_key.to_bytes()),
        encoded_secret_key: hex::encode(secret_key.to_bytes()),
        encoded_target_pub_key: encoded_target_pub_key,
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
    let pub_key_vec = Vec::new();
    pub_key_vec.push(DidCommPubKey {
        id: format!("{}#key-1", from_did),
        r#type: [
            String::from("Ed25519VerificationKey2018"),
        ].to_vec(),
        public_key_base_58: format!("{}", public_key_encoded),
    });

    let service_vec = Vec::new();
    service_vec.push(DidCommService {
        id: format!("{}#didcomm", from_did),
        r#type: String::from("did-communication"),
        priority: 0,
        service_endpoint: format!("{}", service_endpoint),
        recipientKeys: [
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
        &encoded_keypair.encoded_pub_key,
        from_service_endpoint,
    );
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let service_id = format!("{0}#key-1", from_did);
    let exchange_request: MessageWithBody<DidcommObj> = MessageWithBody {
        id: Some(thread_id),
        pthid: Some(format!("{}#key-1", thread_id)),
        thid: Some(service_id),
        from: String::from(from_did),
        to: [String::from(to_did)].to_vec(),
        body: did_comm_obj,
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
    let encoded_keypair = save_com_keypair(
        from_did,
        to_did,
        &keypair.secret,
        &keypair.public,
        &None,
    )?;
    let metadata = serde_json::to_string(&encoded_keypair)?;
    let request_message = get_request_message(&from_did, to_did, "", encoded_keypair)?;

    return get_step_output_decrypted(
        &metadata,
        &serde_json::to_string(&request_message)?,
    );
}

pub fn receive_request(message: &str) -> StepResult {
    let parsed_message: MessageWithBody<DidcommObj> = serde_json::from_str(message)?;
    println!("---------------------------");
    let from_did = parsed_message.from.as_ref().ok_or("from is required")?;
    let to_vec = parsed_message.to.as_ref().ok_or("to is required")?;
    let to_did = &to_vec[0];
    let thread_id = parsed_message.other.get("thid").ok_or("thid is missing")?;
    println!("thread_id: {}", thread_id);
    let didcomm_obj: DidcommObj = parsed_message.body;
    let pub_key_hex = &didcomm_obj.public_key[0].public_key_base_58;
    let service_endpoint = &didcomm_obj.service[0].service_endpoint;

    println!("from_did: {}", from_did);
    println!("to_did: {}", to_did);
    println!("pub_key_hex: {}", pub_key_hex);
    println!("service_endpoint: {}", service_endpoint);

    return get_step_output_decrypted("{}", message);
}
