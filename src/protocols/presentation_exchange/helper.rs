extern crate jsonpath_lib as jsonpath;
use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    datatypes::MessageWithBody,
    protocols::presentation_exchange::datatypes::{
        PresentationExchangeData,
        PresentationExchangeInfo,
        PRESENTATION_EXCHANGE_PROTOCOL_URL,
    },
};

/// Specifies all possible message directions.
#[derive(PartialEq)]
pub enum PresentationExchangeType {
    RequestPresentation,
    ProposePresentation,
    Presentation,
}

/// Constructs a new Presentation Exchange message, including the PresentationExchangeData req as message body.
///
/// # Arguments
/// * `step_type` - step to build the message type (request, response)
/// * `from_did` - DID that sends the message
/// * `to_did` - DID that receives the message
/// * `presentation_exchange_data` - request for credential
/// * `thid` - thid for Issue Credential exchange
///
/// # Returns
/// * `MessageWithBody<PresentationExchangeData>` - constructed Credential request Object, ready to be sent
pub fn get_presentation_exchange_message(
    step_type: PresentationExchangeType,
    from_did: &str,
    to_did: &str,
    presentation_exchange_data: PresentationExchangeData,
    thid: &str,
) -> Result<MessageWithBody<PresentationExchangeData>, Box<dyn std::error::Error>> {
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let step_name = match step_type {
        PresentationExchangeType::RequestPresentation => "request-presentation",
        PresentationExchangeType::ProposePresentation => "propose-presentation",
        PresentationExchangeType::Presentation => "presentation",
    };
    let exchange_request: MessageWithBody<PresentationExchangeData> = MessageWithBody {
        body: Some(presentation_exchange_data),
        created_time: None,
        expires_time: None,
        from: Some(String::from(from_did)),
        id: Some(String::from(&thread_id)),
        other: HashMap::new(),
        pthid: Some(format!("{}#presentation-exchange", thread_id)),
        r#type: format!("{}/{}", PRESENTATION_EXCHANGE_PROTOCOL_URL, step_name),
        thid: Some(thid.to_string()),
        to: Some([String::from(to_did)].to_vec()),
    };

    Ok(exchange_request)
}

/// Takes a Presentation Exchange message and extracts all necessary information to process it during request /
/// response.
///
/// # Arguments
/// * `message` - Presentation Exchange message with PresentationExchange response as body
///
/// # Returns
/// * `PresentationExchangeInfo` - necessary information
pub fn get_presentation_exchange_info_from_message(
    message: MessageWithBody<PresentationExchangeData>,
) -> Result<PresentationExchangeInfo, Box<dyn std::error::Error>> {
    let from_did = message.from.ok_or("from is required")?;
    let to_vec = message.to.ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from("No Credential data was sent."));
    }
    let to_did = &to_vec[0];
    let presentation_exchange_data: PresentationExchangeData =
        message.body.ok_or("body is required.")?;
    let msg_type = message.r#type;

    Ok(PresentationExchangeInfo {
        r#type: msg_type,
        from: Some(from_did),
        to: Some(String::from(to_did)),
        presentation_exchange_data: Some(presentation_exchange_data),
    })
}

/// Validates received presentation credentials against the requested presentation_definition
/// response.
///
/// # Arguments
/// * `request_presentation` - Presentation request
/// * `received_presentation` - Received Presentation from Holder
///
/// # Returns
/// * `bool` - returns true if presentation is valid and satisfies constraints otherwise returns false
pub fn validate_presentation_against_credentials(
    request_presentation: PresentationExchangeData,
    received_presentation: PresentationExchangeData,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_json = serde_json::to_value(request_presentation)?;
    let received_json = serde_json::to_value(received_presentation)?;
    let mut request_selector = jsonpath::selector(&request_json);
    let mut received_selector = jsonpath::selector(&received_json);
    let requested_credentials = request_selector("$.request_presentation_attach.data[0].json.presentation_definition.input_descriptors[*].name")?;
    let received_credentials =
        received_selector("$.presentations_attach.data[0].json.verifiable_credential[*].type")?;

    if requested_credentials != received_credentials {
        return Err(Box::from(
            "Received credentials do not match the requested presentation",
        ));
    }

    Ok(())
}
