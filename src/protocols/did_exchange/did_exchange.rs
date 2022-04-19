use crate::{
    db::{read_db, write_db},
    protocols::did_exchange::datatypes::{State, UserType},
};

/// Saves a invite request/response in db for two DIDs (from -> to). Entry key will be
/// did_exchange_{from}_{to}_{state}_{thid}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `thid` - thread id
/// * `exchange_data` - did exchange data
/// * `state` - State
pub fn save_didexchange(
    from_did: &str,
    to_did: &str,
    thid: &str,
    exchange_data: &str,
    state: &State,
) -> Result<(), Box<dyn std::error::Error>> {
    write_db(
        &format!("did_exchange_{}_{}_{}_{}", from_did, to_did, state, thid),
        exchange_data,
    )?;

    Ok(())
}

/// Saves state of Did Exchange protocol for given thid. Entry key will be
/// did_exchange_state_{thid}.
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
        &format!("did_exchange_state_{}_{}", user_type, thid),
        &state.to_string(),
    )?;

    Ok(())
}

/// Retrieves state of Did Exchange protocol for given thid. Entry key will be
/// did_exchange_state_{thid}.
///
/// # Arguments
/// * `thid` - thread id
/// * `user_type` - UserType
///
/// # Returns
/// * `state` - State stored in db.
pub fn get_current_state(
    thid: &str,
    user_type: &UserType,
) -> Result<String, Box<dyn std::error::Error>> {
    let result = read_db(&format!("did_exchange_state_{}_{}", user_type, thid));
    let state = match result {
        Ok(value) => value,
        Err(_) => "Unknown".to_string(),
    };
    Ok(state)
}
