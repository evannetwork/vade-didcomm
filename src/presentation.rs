use crate::{datatypes::PresentationData, db::{read_db, write_db}};

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

    Ok(())
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
) -> Result<PresentationData, Box<dyn std::error::Error>> {
    let presentation = read_db(&format!("present_proof_{}_{}", from_did, to_did))?;
    let presentation_data: PresentationData = serde_json::from_str(&presentation)?;
    Ok(presentation_data)
}
