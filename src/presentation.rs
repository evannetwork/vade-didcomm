use crate::{
    datatypes::{PresentationData, State},
    db::{read_db, write_db},
};

/// Saves a request-presentation/presentation in db for two DIDs (from -> to). Entry key will be
/// present_proof_{from}_{to}_{state}_{thid}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `thid` - thread id
/// * `presentation` - presentation data
pub fn save_presentation(
    from_did: &str,
    to_did: &str,
    thid: &str,
    presentation: &str,
    state: &State,
) -> Result<(), Box<dyn std::error::Error>> {
    write_db(
        &format!("present_proof_{}_{}_{}_{}", from_did, to_did, state, thid),
        presentation,
    )?;

    Ok(())
}

/// Retrieves state presentation data from the db for two DIDs (from -> to). Entry key will be
/// present_proof_{from}_{to}_{state}_{thid}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `thid` - thread id
/// * `state` - state
/// # Returns
/// * `Presentation` - presetation data stored in db.
pub fn _get_presentation(
    from_did: &str,
    to_did: &str,
    thid: &str,
    state: &State,
) -> Result<PresentationData, Box<dyn std::error::Error>> {
    let presentation = read_db(&format!("present_proof_{}_{}_{}_{}", from_did, to_did, state, thid))?;
    let presentation_data: PresentationData = serde_json::from_str(&presentation)?;
    Ok(presentation_data)
}


/// Saves state of Present_Proof protocol for given thid. Entry key will be
/// present_proof_state_{thid}.
///
/// # Arguments
/// * `state` - State
/// * `thid` - thread id
pub fn save_state(
    thid: &str,
    state: &State,
) -> Result<(), Box<dyn std::error::Error>> {
    write_db(
        &format!("present_proof_state_{}", thid),
        &state.to_string(),
    )?;

    Ok(())
}
/// Retrieves state of Present_Proof protocol for given thid. Entry key will be
/// present_proof_state_{thid}.
///
/// # Arguments
/// * `thid` - thread id
/// # Returns
/// * `state` - State stored in db.
pub fn get_current_state(
    thid: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let state = read_db(&format!("present_proof_state_{}", thid))?;
    Ok(state)
}