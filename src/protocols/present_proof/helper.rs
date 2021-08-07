use std::collections::HashMap;

use uuid::Uuid;

use crate::datatypes::{
    PresentProofInfo,
    MessageWithBody,
    PRESENT_PROOF_PROTOCOL_URL,
};

/// Specifies all possible message directions.
#[derive(PartialEq)]
pub enum PresentProofType {
    REQUEST_PRESENTATION,
    PRESENTATION,
    PROPOSE_PRESENTATION,
    ACK,
}



/// Constructs a new Request Presentation message, including the Presentation req as message body.
///
/// # Arguments
/// * `step_type` - step to build the message type (request, response)
/// * `from_did` - DID that sends the message
/// * `to_did` - DID that receives the message
/// * `from_service_endpoint` - url where the user can be reached
/// * `request_presentation` - request for presentation
///
/// # Returns
/// * `MessageWithBody<String>` - constructed Presentation request Object, ready to be sent
pub fn get_present_proof_message(
    step_type: PresentProofType,
    from_did: &str,
    to_did: &str,
    from_service_endpoint: &str,
    request_presentation: &str,
) -> Result<MessageWithBody<String>, Box<dyn std::error::Error>> {
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let service_id = format!("{0}#key-1", from_did);
    let step_name = match step_type {
        PresentProofType::REQUEST_PRESENTATION => "request-presentation",
        PresentProofType::PROPOSE_PRESENTATION => "propose-presentation",
        PresentProofType::PRESENTATION => "presentation",
        PresentProofType::ACK => "ack",
    };
    let exchange_request: MessageWithBody<String> = MessageWithBody {
        body: Some(request_presentation.to_string()),
        created_time: None,
        expires_time: None,
        from: Some(String::from(from_did)),
        id: Some(String::from(&thread_id)),
        other: HashMap::new(),
        pthid: Some(format!("{}#key-1", String::from(thread_id))),
        r#type: format!("{}/{}", PRESENT_PROOF_PROTOCOL_URL, step_name),
        thid: Some(service_id),
        to: Some([String::from(to_did)].to_vec()),
    };

    return Ok(exchange_request);
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
    message: MessageWithBody<String>,
) -> Result<PresentProofInfo, Box<dyn std::error::Error>> {
    let from_did = message.from.ok_or("from is required")?;

    let to_vec = message.to.ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from(
            "No Presentation data was sent from Prover.",
        ));
    }
    let to_did = &to_vec[0];
    let presentation: String = message.body.ok_or("body is required")?;
   
    return Ok(PresentProofInfo {
        from: String::from(from_did),
        to: String::from(to_did),
        presentation_data: String::from(presentation),
    });
}
