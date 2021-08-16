use crate::{db::{write_db}};

/// Saves a request-presentation/presentation in db for two DIDs (from -> to). Entry key will be
/// present_proof_{from}_{to}.
///
/// # Arguments
/// * `from_did` - from DID
/// * `to_did` - to DID as string
/// * `thid` - thread id as string
/// * `presentation` - presentation data 
pub fn save_presentation(
    from_did: &str,
    to_did: &str,
    thid: &str,
    presentation: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    write_db(
        &format!("present_proof_{}_{}_{}", from_did, to_did, thid),
        presentation,
    )?;

    Ok(())
}
