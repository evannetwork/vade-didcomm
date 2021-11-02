use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub const PRESENTATION_EXCHANGE_PROTOCOL_URL: &str =
    "https://identity.foundation/presentation-exchange/spec/v1.0.0";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresentationExchangeInfo {
    pub r#type: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub presentation_exchange_data: Option<PresentationExchangeData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialSubject {
    pub id: String,
    pub data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Proof {
    pub r#type: String,
    pub created: String,
    pub proof_purpose: String,
    pub verification_method: String,
    pub jws: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VerifiableCredential {
    pub context: String,
    pub id: String,
    pub r#type: String,
    pub issuer: String,
    pub issuance_date: String,
    pub credential_subject: CredentialSubject,
    pub proof: Proof,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationSubmission {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor_map: Option<Vec<DescriptorMap>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DescriptorMap {
    pub id: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_nested: Option<Box<DescriptorMap>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Options {
    pub challenge: String,
    pub domain: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationDefinition {
    pub id: String,
    pub input_descriptors: Vec<InputDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Format>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_requirements: Option<Vec<SubmissionRequirement>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmissionRequirement {
    pub rule: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_nested: Option<Vec<SubmissionRequirement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Format {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attach_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<Alg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_vc: Option<Alg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_vp: Option<Alg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldp_vc: Option<ProofType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldp_vp: Option<ProofType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldp: Option<ProofType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputDescriptor {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Format>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub schema: Vec<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Constraints>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Constraints {
    pub fields: Vec<Field>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // Values should be required, preferred
    pub limit_disclosure: Option<ValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<Status>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // Values should be required, preferred
    pub subject_is_issuer: Option<ValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_holder: Option<Vec<GenericObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_subject: Option<Vec<GenericObject>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenericObject {
    pub field_id: Vec<String>,
    // Values should be required, preferred
    pub directive: Option<ValueType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Status {
    #[serde(skip_serializing_if = "Option::is_none")]
    // Values should be required, allowed or disallowed
    pub active: Option<ValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // Values should be required, allowed or disallowed
    pub suspended: Option<ValueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // Values should be required, allowed or disallowed
    pub revoked: Option<ValueType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Field {
    pub path: Vec<String>,
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // Value should be either required or preferred
    pub predicate: Option<ValueType>,
    pub filter: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Filter {
    pub r#type: String,
    pub pattern: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Schema {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Alg {
    pub alg: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofType {
    pub proof_type: Vec<String>,
}

/// PresentationExchangeData struct is the general structure which contains all optional fields
/// required for all messages of Presentation Exchange protocol.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationExchangeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Vec<Format>>,
    pub state: State,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal_attach: Option<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_presentation_attach: Option<Attachment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentations_attach: Option<Attachment>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attachment {
    pub id: String,
    pub mime_type: String,
    pub data: Vec<Data>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    pub json: JsonData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JsonData {
    // input_descriptors are required only for propose_presentation message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_descriptors: Option<Vec<InputDescriptor>>,
    // context, type, presentation_submission, verifiable_credential and proof are required
    // only for presentation message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_submission: Option<PresentationSubmission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifiable_credential: Option<Vec<VerifiableCredential>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<Proof>,
    // options and presentation_definition are required for request_presentation message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Options>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_definition: Option<PresentationDefinition>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum State {
    SendPresentationRequest,
    ReceivePresentatonRequest,
    SendProposePresentation,
    ReceiveProposePresentation,
    SendPresentation,
    ReceivePresentation,
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
            "SendPresentationRequest" => Ok(State::SendPresentationRequest),
            "ReceivePresentatonRequest" => Ok(State::ReceivePresentatonRequest),
            "SendProposePresentation" => Ok(State::SendProposePresentation),
            "ReceiveProposePresentation" => Ok(State::ReceiveProposePresentation),
            "SendPresentation" => Ok(State::SendPresentation),
            "ReceivePresentation" => Ok(State::ReceivePresentation),
            _ => Ok(State::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserType {
    Verifier,
    Holder,
    None,
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ValueType {
    Required,
    Preferred,
    Allowed,
    DisAllowed,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
