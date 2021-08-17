use std::collections::HashMap;

use uuid::Uuid;

use crate::datatypes::{MessageWithBody, PresentProofReq, PresentationData};

pub const PRESENT_PROOF_PROTOCOL_URL: &str = "https://didcomm.org/present-proof/1.0";

/// Specifies all possible message directions.
#[derive(PartialEq)]
pub enum PresentProofType {
    RequestPresentation,
    Presentation,
    ProposePresentation,
}

/// Constructs a new Request Presentation message, including the Presentation req as message body.
///
/// # Arguments
/// * `step_type` - step to build the message type (request, response)
/// * `from_did` - DID that sends the message
/// * `to_did` - DID that receives the message
/// * `request_presentation` - request for presentation
/// * `thid` - thid for Present-Proof exchange
///
/// # Returns
/// * `MessageWithBody<PresentationData>` - constructed Presentation request Object, ready to be sent
pub fn get_present_proof_message(
    step_type: PresentProofType,
    from_did: &str,
    to_did: &str,
    presentation_data: PresentationData,
    thid: &str,
) -> Result<MessageWithBody<PresentationData>, Box<dyn std::error::Error>> {
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let step_name = match step_type {
        PresentProofType::RequestPresentation => "request-presentation",
        PresentProofType::ProposePresentation => "propose-presentation",
        PresentProofType::Presentation => "presentation",
    };
    let exchange_request: MessageWithBody<PresentationData> = MessageWithBody {
        body: Some(presentation_data),
        created_time: None,
        expires_time: None,
        from: Some(String::from(from_did)),
        id: Some(String::from(&thread_id)),
        other: HashMap::new(),
        pthid: Some(format!("{}#present-proof", thread_id)),
        r#type: format!("{}/{}", PRESENT_PROOF_PROTOCOL_URL, step_name),
        thid: Some(thid.to_string()),
        to: Some([String::from(to_did)].to_vec()),
    };

    Ok(exchange_request)
}

/// Takes an PresentProof message and extracts all necessary information to process it during request /
/// response.
///
/// # Arguments
/// * `message` - PresentProof message with proover presentation response as body
///
/// # Returns
/// * `PresentProofInfo` - necessary information
pub fn get_present_proof_info_from_message(
    message: MessageWithBody<PresentationData>,
) -> Result<PresentProofReq, Box<dyn std::error::Error>> {
    let from_did = message.from.ok_or("from is required")?;
    let to_vec = message.to.ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from("No Presentation data was sent from Prover."));
    }
    let to_did = &to_vec[0];
    let presentation: PresentationData = message.body.ok_or("body is required")?;
    let msg_type = message.r#type;

    Ok(PresentProofReq {
        r#type: msg_type,
        from: Some(from_did),
        to: Some(String::from(to_did)),
        presentation_data: Some(presentation),
    })
}
