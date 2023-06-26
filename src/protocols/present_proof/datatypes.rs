use std::fmt;

use serde::{Deserialize, Serialize};

use crate::datatypes::Data;

pub const PRESENT_PROOF_PROTOCOL_URL: &str = "https://didcomm.org/present-proof/1.0";
pub const PROPOSAL_PROTOCOL_URL: &str =
    "https://didcomm.org/present-proof/1.0/presentation-preview";

pub trait MessageData {}

/// data structure for presentation request
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Vec<PresentationAttach>,
}
impl MessageData for RequestData {}

/// data structure with actual presentation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Vec<PresentationAttach>,
}
impl MessageData for PresentationData {}

/// data structure for proposing a new presentation request
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProposalData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "proposals~attach")]
    pub proposals_attach: Vec<PresentationAttach>,
}
impl MessageData for ProposalData {}

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
impl MessageData for ProblemReportData {}

// properties for Ack messages that are not part of the default DIDComm message set
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AckData {
    pub status: AckStatus,
}
impl MessageData for AckData {}

/// PresentationAttach contains all the fields required for
/// propose-presentation, request-presentation, and presentation steps.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationAttach {
    pub id: String,
    #[serde(rename = "mime-type")]
    pub mime_type: String,
    pub data: Data,
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
        write!(f, "{self:?}")
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
        write!(f, "{self:?}")
    }
}
