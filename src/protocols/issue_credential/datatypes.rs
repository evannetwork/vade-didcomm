use std::fmt;

use serde::{Deserialize, Serialize};

pub const ISSUE_CREDENTIAL_PROTOCOL_URL: &str = "https://didcomm.org/issue-credential/1.0";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IssuerCredentialReq {
    pub r#type: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub credential_data: Option<CredentialData>,
}

/// CredentialAttach struct contains common fields which are required by
/// offer-credential/request-credential/issue-credential messages for attachment.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialAttach {
    pub id: String,
    pub mime_type: String,
    pub data: String,
}

/// CredentialProposal struct contains fields required by propose-credential message.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialProposal {
    pub id: String,
    pub comment: String,
    pub schema_issuer_did: String,
    pub schema_id: String,
    pub schema_name: String,
    pub schema_version: String,
    pub cred_def_id: String,
    pub issuer_did: String,
}

/// Attribute struct is required for Credential Preview.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub mime_type: String,
    pub value: String,
}

/// CredentialPreview struct contains fields required for offer-credential and propose-credential message.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialPreview {
    pub r#type: String,
    pub attributes: Vec<Attribute>,
}

/// CredentialData struct is the general structure which contains all optional fields
/// required for all messages of Issue Credential protocol.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialData {
    #[serde(skip_serializing_if = "Option::is_none")]
    // credential_preview is sent only with offer-credential, propose-credential
    pub credential_preview: Option<CredentialPreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_attach: Option<Vec<CredentialAttach>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_proposal: Option<CredentialProposal>,
}

// properties for ProblemReport messages that are not part of the default DIDComm message set
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProblemReportData {
    pub user_type: UserType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_items: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_retries: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#where: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noticed_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation_uri: Option<String>,
}

/// Problem report structure contains fields which are required for reporting problem
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProblemReport {
    pub r#type: String,
    pub from: Option<String>,
    pub to: Option<Vec<String>>,
    pub id: String,
    pub thid: Option<String>,
    pub body: ProblemReportData,
}

// properties for Ack messages that are not part of the default DIDComm message set
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AckData {
    pub status: AckStatus,
    pub user_type: UserType,
}

/// Ack structure contains fields which are sent as acknowledgment of received credential
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ack {
    pub from: Option<String>,
    pub to: Option<Vec<String>>,
    pub r#type: String,
    pub id: String,
    pub thid: Option<String>,
    pub body: AckData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AckStatus {
    OK,
    FAIL,
    PENDING,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum State {
    SendProposeCredential,
    ReceiveProposeCredential,
    SendOfferCredential,
    ReceiveOfferCredential,
    SendRequestCredential,
    ReceiveRequestCredential,
    SendIssueCredential,
    ReceiveIssueCredential,
    ProblemReported,
    Acknowledged,
    Unknown,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for State {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SendProposeCredential" => Ok(State::SendProposeCredential),
            "ReceiveProposeCredential" => Ok(State::ReceiveProposeCredential),
            "SendOfferCredential" => Ok(State::SendOfferCredential),
            "ReceiveOfferCredential" => Ok(State::ReceiveOfferCredential),
            "SendRequestCredential" => Ok(State::SendRequestCredential),
            "ReceiveRequestCredential" => Ok(State::ReceiveRequestCredential),
            "SendIssueCredential" => Ok(State::SendIssueCredential),
            "ReceiveIssueCredential" => Ok(State::ReceiveIssueCredential),
            "ProblemReported" => Ok(State::ProblemReported),
            "Acknowledged" => Ok(State::Acknowledged),
            _ => Ok(State::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserType {
    Issuer,
    Holder,
    None,
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
