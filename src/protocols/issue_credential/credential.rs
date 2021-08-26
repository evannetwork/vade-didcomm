use crate::{
    db::{read_db, write_db},
    protocols::issue_credential::datatypes::{CredentialData, State, UserType},
};

/// Saves a state of credential (request/offer/issue/propose) in db for two DIDs (from -> to). Entry key will be
/// issue_credential_{from}_{to}_{state}_{thid}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `thid` - thread id
/// * `credential` - credential data
/// * `state` - State
pub fn save_credential(
    from_did: &str,
    to_did: &str,
    thid: &str,
    credential: &str,
    state: &State,
) -> Result<(), Box<dyn std::error::Error>> {
    write_db(
        &format!(
            "issue_credential_{}_{}_{}_{}",
            from_did, to_did, state, thid
        ),
        credential,
    )?;

    Ok(())
}

/// Retrieves state of credential (request/offer/issue/propose) from the db for two DIDs (from -> to). Entry key will be
/// issue_credential_{from}_{to}_{state}_{thid}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `thid` - thread id
/// * `state` - state
/// # Returns
/// * `Credential` - credential data stored in db.
#[allow(dead_code)]
pub fn get_credential(
    from_did: &str,
    to_did: &str,
    thid: &str,
    state: &State,
) -> Result<CredentialData, Box<dyn std::error::Error>> {
    let credential = read_db(&format!(
        "issue_credential_{}_{}_{}_{}",
        from_did, to_did, state, thid
    ))?;
    let credential_data: CredentialData = serde_json::from_str(&credential)?;
    Ok(credential_data)
}

/// Saves state of Issue Credential protocol for given thid. Entry key will be
/// issue_credential_state_{thid}.
///
/// # Arguments
/// * `state` - State
/// * `thid` - thread id
/// * `user_type` - UserType
pub fn save_state(
    thid: &str,
    state: &State,
    user_type: &UserType,
) -> Result<(), Box<dyn std::error::Error>> {
    write_db(
        &format!("issue_credential_state_{}_{}", user_type, thid),
        &state.to_string(),
    )?;

    Ok(())
}
/// Retrieves state of Issue Credential protocol for given thid. Entry key will be
/// issue_credential_state_{user_type}_{thid}.
///
/// # Arguments
/// * `thid` - thread id
/// * `user_type` - UserType
/// # Returns
/// * `state` - State stored in db.
pub fn get_current_state(
    thid: &str,
    user_type: &UserType,
) -> Result<String, Box<dyn std::error::Error>> {
    let result = read_db(&format!("issue_credential_state_{}_{}", user_type, thid));
    let state = match result {
        Ok(value) => value,
        Err(_) => "Unknown".to_string(),
    };
    Ok(state)
}
