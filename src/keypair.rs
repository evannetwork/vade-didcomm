use serde::{Deserialize, Serialize};

use crate::{read_db, utils::SyncResult, write_db};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommKeyPair {
    pub pub_key: String,
    pub secret_key: String,
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
) -> SyncResult<CommKeyPair> {
    let comm_keypair = CommKeyPair {
        pub_key: String::from(pub_key),
        secret_key: String::from(secret_key),
        target_pub_key: target_pub_key.unwrap_or(String::from("")),
        target_service_endpoint: service_endpoint.unwrap_or(String::from("")),
    };

    let _ = write_db(
        &format!("comm_keypair_{}_{}", from_did, to_did),
        &serde_json::to_string(&comm_keypair)?,
    );

    return Ok(comm_keypair);
}

pub fn get_com_keypair(from_did: &str, to_did: &str) -> SyncResult<CommKeyPair> {
    let db_result = read_db(&format!("comm_keypair_{}_{}", from_did, to_did))?;
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    return Ok(comm_keypair);
}
