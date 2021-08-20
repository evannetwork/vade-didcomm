use crate::datatypes::MessageWithBody;
use crate::protocols::issue_credential::datatypes::{
    IssuerCredentialReq, CredentialData, ISSUE_CREDENTIAL_PROTOCOL_URL,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Specifies all possible message directions.
#[derive(PartialEq)]
pub enum IssueCredentialType {
    ProposeCredential,
    OfferCredential,
    RequestCredential,
    IssueCredential,
}

/// Constructs a new Request Credential message, including the CredentialData req as message body.
///
/// # Arguments
/// * `step_type` - step to build the message type (request, response)
/// * `from_did` - DID that sends the message
/// * `to_did` - DID that receives the message
/// * `credential_data` - request for credential
/// * `thid` - thid for Issue Credential exchange
///
/// # Returns
/// * `MessageWithBody<CredentialData>` - constructed Credential request Object, ready to be sent
pub fn get_issue_credential_message(
    step_type: IssueCredentialType,
    from_did: &str,
    to_did: &str,
    credential_data: CredentialData,
    thid: &str,
) -> Result<MessageWithBody<CredentialData>, Box<dyn std::error::Error>> {
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let step_name = match step_type {
        IssueCredentialType::ProposeCredential => "propose-credential",
        IssueCredentialType::IssueCredential => "issue-credential",
        IssueCredentialType::OfferCredential => "offer-credential",
        IssueCredentialType::RequestCredential => "request-credential",
    };
    let exchange_request: MessageWithBody<CredentialData> = MessageWithBody {
        body: Some(credential_data),
        created_time: None,
        expires_time: None,
        from: Some(String::from(from_did)),
        id: Some(String::from(&thread_id)),
        other: HashMap::new(),
        pthid: Some(format!("{}#issue-credential", thread_id)),
        r#type: format!("{}/{}", ISSUE_CREDENTIAL_PROTOCOL_URL, step_name),
        thid: Some(thid.to_string()),
        to: Some([String::from(to_did)].to_vec()),
    };

    Ok(exchange_request)
}

/// Takes an Issue Credential message and extracts all necessary information to process it during request /
/// response.
///
/// # Arguments
/// * `message` - Issue Credential message with proover CredentialData response as body
///
/// # Returns
/// * `IssuerCredentialReq` - necessary information
pub fn get_issue_credential_info_from_message(
    message: MessageWithBody<CredentialData>,
) -> Result<IssuerCredentialReq, Box<dyn std::error::Error>> {
    let from_did = message.from.ok_or("from is required")?;
    let to_vec = message.to.ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from("No Credential data was sent."));
    }
    let to_did = &to_vec[0];
    let credential_data: CredentialData = message.body.ok_or("body is required.")?;
    let msg_type = message.r#type;

    Ok(IssuerCredentialReq {
        r#type: msg_type,
        from: Some(from_did),
        to: Some(String::from(to_did)),
        credential_data: Some(credential_data),
    })
}
