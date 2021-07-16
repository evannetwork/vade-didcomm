use crate::{
    datatypes::CommKeyPair,
    rocks_db::{read_db, write_db},
    utils::SyncResult,
};

/// Saves a communication keypair within the rocks.db for two dids (from -> to). Entry key will be
/// comm_keypair_{from}_{to}.
///
/// # Arguments
/// * `from_did` - from did
/// * `to_did` - to did as string
/// * `pub_key` - pub key of the active did to communicate with the target did
/// * `secret_key` - secret key of the active did to encrypt message for the target did
/// * `target_pub_key` - pub key of the target did (optional nullable, default will be empty string)
/// * `service_endpoint` - url, where the target did can be reached (optional nullable, default will be empty string)
///
/// # Returns
/// * `CommKeyPair` - new instance of the comm key pair
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

    write_db(
        &format!("comm_keypair_{}_{}", from_did, to_did),
        &serde_json::to_string(&comm_keypair)?,
    )?;

    return Ok(comm_keypair);
}

/// Loads a communication keypair from the rocks db for two dids (from -> to). Entry key will be
/// comm_keypair_{from}_{to}.
///
/// # Arguments
/// * `from_did` - from did
/// * `to_did` - to did as string
///
/// # Returns
/// * `CommKeyPair` - new instance of the comm key pair
pub fn get_com_keypair(from_did: &str, to_did: &str) -> SyncResult<CommKeyPair> {
    let db_result = read_db(&format!("comm_keypair_{}_{}", from_did, to_did))?;
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    return Ok(comm_keypair);
}
