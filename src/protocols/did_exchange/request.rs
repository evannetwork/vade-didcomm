use uuid::Uuid;
use k256::elliptic_curve::rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{Message, protocol::DID_EXCHANGE_PROTOCOL_URL, utils::SyncResult, write_db};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommKeyPair {
    pub encoded_pub_key: String,
    pub encoded_secret_key: String,
    pub encoded_target_pub_key: String,
}

pub fn get_encoded_pub_key() {

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
) -> String {
    return format!(
        r#"{{
            "@context": "https://w3id.org/did/v1",
            "id": "{0}",
            "publicKey": [{{
                "id": "{0}#key-1",
                "type": [
                  "Ed25519VerificationKey2018"
                ],
                "publicKeyBase58": "{1}"
            }}],
            "authentication": [
              "{0}#key-1"
            ],
            "service": [{{
                "id": "{0}#didcomm",
                "type": "did-communication",
                "priority": 0,
                "serviceEndpoint": "{2}",
                "recipientKeys": ["{1}"]
            }}]
          }}"#,
          from_did,
          public_key_encoded,
          service_endpoint,
    );
}

pub fn get_request_message(
    from_did: &str,
    to_did: &str,
    from_service_endpoint: &str,
) -> SyncResult<String> {
    let keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
    let encoded_keypair = save_com_keypair(
        from_did,
        to_did,
        &keypair.secret,
        &keypair.public,
        &None,
    )?;
    let message_payload = get_com_did_obj(
        from_did,
        &encoded_keypair.encoded_pub_key,
        from_service_endpoint,
    );
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let service_id = format!("{0}#key-1", from_did);
    let exchange_request =  format!(
        r#"{{
            "id": "{}",
            "pthid": "{}#key-1",
            "thid": "{}",
            "from": "{}",
            "to": ["{}"],
            "body": {},
            "type": "{}/request",
        }}"#,
        thread_id,
        thread_id,
        service_id,
        from_did,
        to_did,
        message_payload,
        DID_EXCHANGE_PROTOCOL_URL,
    );

    return Ok(exchange_request);
}

pub fn send_request(message: &mut Message, encrypt: &mut bool) -> SyncResult<String> {
    let from = message.from.as_ref().ok_or("from is required")?;
    let to_vec = message.to.as_ref().ok_or("to is required")?;
    let to = &to_vec[0];

    // TODO: replace empty string with from_service_endpoint
    message.body = get_request_message(&from, to, "")?;
    *encrypt = false;

    return Ok(String::from("{}"));
}

pub fn receive_request(message: &mut Message, _encrypt: &mut bool) -> SyncResult<String> {
    return Ok(String::from("{}"));
}
