use std::fmt;

use serde::{Deserialize, Serialize};

pub const PRESENT_PROOF_PROTOCOL_URL: &str = "https://didcomm.org/present-proof/1.0";

/// This structure is required to be present in all the steps of Present-Proof protocol for send and receive directions.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresentProofReq {
    pub r#type: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub presentation_data: Option<PresentationData>,
}

/// PresentationAttach contains all the fields required for request-presentation and presentation steps.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationAttach {
    pub r#type: String,
    pub id: String,
    #[serde(rename = "mime-type")]
    pub mime_type: String,
    pub data: String,
}

/// Presentation preview structure is sent by prover to propose alternate presentation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationPreview {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<Vec<Attribute>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predicate: Option<Vec<Predicate>>,
}

/// Attributes structure for PresentationPreview request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub cred_def_id: String,
    #[serde(rename = "mime-type")]
    pub mime_type: String,
    pub value: String,
    pub referent: String,
}

/// Predicate structure for PresentationPreview request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Predicate {
    pub name: String,
    pub cred_def_id: String,
    pub predicate: String,
    pub threshold: u64,
}

/// PresentationData structure contains optional fields to be exchanged for all the steps of Present-Proof steps.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_attach: Option<Vec<PresentationAttach>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_proposal: Option<PresentationPreview>,
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

/// Ack structure contains fields which are sent to
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
    PresentationRequested,
    PresentationRequestReceived,
    PresentationReceived,
    PresentationSent,
    PresentationProposed,
    PresentationProposalReceived,
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
            "PresentationRequested" => Ok(State::PresentationRequested),
            "PresentationRequestReceived" => Ok(State::PresentationRequestReceived),
            "PresentationReceived" => Ok(State::PresentationReceived),
            "PresentationSent" => Ok(State::PresentationSent),
            "PresentationProposed" => Ok(State::PresentationProposed),
            "PresentationProposalReceived" => Ok(State::PresentationProposalReceived),
            "ProblemReported" => Ok(State::ProblemReported),
            "Acknowledged" => Ok(State::Acknowledged),
            _ => Ok(State::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserType {
    Prover,
    Verifier,
    None,
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
