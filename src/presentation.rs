use crate::{
    db::{read_db, write_db},
};

/// Saves a request-presentation/presentation in db for two DIDs (from -> to). Entry key will be
/// present_proof_{from}_{to}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `presentation` - presentation data 
pub fn save_presentation(
    from_did: &str,
    to_did: &str,
    presentation: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    write_db(
        &format!("present_proof_{}_{}", from_did, to_did),
        presentation,
    )?;

    return Ok(());
}

/// Retrieves presentation data from the db for two DIDs (from -> to). Entry key will be
/// present_proof_{from}_{to}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
///
/// # Returns
/// * `Presentation` - presetation data stored in db.
pub fn get_presentation(
    from_did: &str,
    to_did: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let presentation = read_db(&format!("present_proof_{}_{}", from_did, to_did))?;
    return Ok(presentation);
}
