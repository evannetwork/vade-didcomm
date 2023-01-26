#[cfg(feature = "state_storage")]
use crate::db::write_db;
use crate::{datatypes::CommKeyPair, db::read_db};

/// Saves a communication keypair within db for two DIDs (from -> to). Entry key will be
/// comm_keypair_{from}_{to}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `pub_key` - pub key of the active did to communicate with the target did
/// * `secret_key` - secret key of the active did to encrypt message for the target did
/// * `target_pub_key` - pub key of the target did (optional nullable, default will be empty string)
/// * `service_endpoint` - url, where the target did can be reached (optional nullable, default will be empty string)
///
/// # Returns
/// * `CommKeyPair` - new instance of the comm key pair
#[allow(clippy::too_many_arguments)]
pub fn save_com_keypair(
    #[allow(unused_variables)] // may not be used, depending on feature setup
    from_did: &str,
    #[allow(unused_variables)] // may not be used, depending on feature setup
    to_did: &str,
    key_agreement_key: &str,
    target_key_agreement_key: &str,
    pub_key: &str,
    secret_key: &str,
    target_pub_key: Option<String>,
    service_endpoint: Option<String>,
) -> Result<CommKeyPair, Box<dyn std::error::Error>> {
    let comm_keypair = CommKeyPair {
        pub_key: String::from(pub_key),
        secret_key: String::from(secret_key),
        key_agreement_key: String::from(key_agreement_key),
        target_key_agreement_key: String::from(target_key_agreement_key),
        target_pub_key: target_pub_key.unwrap_or_else(|| String::from("")),
        target_service_endpoint: service_endpoint.unwrap_or_else(|| String::from("")),
    };

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            write_db(
                &format!("comm_keypair_{}_{}", from_did, to_did),
                &serde_json::to_string(&comm_keypair)?,
            )?;

            write_db(
                &format!("key_agreement_key_{}", key_agreement_key),
                &serde_json::to_string(&comm_keypair)?,
            )?;
        } else { }
    }

    Ok(comm_keypair)
}

/// Loads a communication keypair from the db for two DIDs (from -> to). Entry key will be
/// comm_keypair_{from}_{to}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
///
/// # Returns
/// * `CommKeyPair` - new instance of the comm key pair
pub fn get_com_keypair(
    from_did: &str,
    to_did: &str,
) -> Result<CommKeyPair, Box<dyn std::error::Error>> {
    let db_result = read_db(&format!("comm_keypair_{}_{}", from_did, to_did))?;
    log::debug!(
        "receiving key: comm_keypair_{}_{} with data: {}",
        from_did,
        to_did,
        db_result,
    );
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    Ok(comm_keypair)
}

/// Loads a communication keypair from the db for two DIDs (from -> to). Entry key will be
/// comm_keypair_{from}_{to}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
///
/// # Returns
/// * `CommKeyPair` - new instance of the comm key pair
pub fn get_key_agreement_key(
    key_agreement_key: &str,
) -> Result<CommKeyPair, Box<dyn std::error::Error>> {
    let db_result = read_db(&format!("key_agreement_key_{}", key_agreement_key))?;
    log::debug!(
        "receving key: key_agreement_key_{} with data: {}",
        key_agreement_key,
        db_result,
    );
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    Ok(comm_keypair)
}
