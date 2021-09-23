use crate::{
    db::{read_db, write_db},
    protocols::presentation_exchange::datatypes::{PresentationExchangeData, State, UserType},
};

/// Saves a state of presentation exchange (request/propose/presentation) in db for two DIDs (from -> to). Entry key will be
/// presentation_exchange_{from}_{to}_{state}_{thid}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `thid` - thread id
/// * `presentation_exchange` - presentation exchange data
/// * `state` - State
pub fn save_presentation_exchange(
    from_did: &str,
    to_did: &str,
    thid: &str,
    presentation_exchange: &str,
    state: &State,
) -> Result<(), Box<dyn std::error::Error>> {
    write_db(
        &format!(
            "presentation_exchange_{}_{}_{}_{}",
            from_did, to_did, state, thid
        ),
        presentation_exchange,
    )?;

    Ok(())
}

/// Retrieves state of presentation exchange (request/propose/presentation) from the db for two DIDs (from -> to). Entry key will be
/// presentation_exchange_{from}_{to}_{state}_{thid}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `thid` - thread id
/// * `state` - state
/// # Returns
/// * `presentation_exchange` - presentation_exchange data stored in db.
#[allow(dead_code)]
pub fn get_presentation_exchange(
    from_did: &str,
    to_did: &str,
    thid: &str,
    state: &State,
) -> Result<PresentationExchangeData, Box<dyn std::error::Error>> {
    let presentation_exchange = read_db(&format!(
        "presentation_exchange_{}_{}_{}_{}",
        from_did, to_did, state, thid
    ))?;
    let presentation_exchange: PresentationExchangeData =
        serde_json::from_str(&presentation_exchange)?;
    Ok(presentation_exchange)
}

/// Saves state of Presentation Exchange protocol for given thid. Entry key will be
/// presentation_exchange_state_{user_type}_{thid}.
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
        &format!("presentation_exchange_state_{}_{}", user_type, thid),
        &state.to_string(),
    )?;

    Ok(())
}
/// Retrieves state of Presentation Exchange protocol for given thid. Entry key will be
/// presentation_exchange_state_{user_type}_{thid}.
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
    let result = read_db(&format!(
        "presentation_exchange_state_{}_{}",
        user_type, thid
    ));
    let state = match result {
        Ok(value) => value,
        Err(_) => "Unknown".to_string(),
    };
    Ok(state)
}
