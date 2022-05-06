use std::fmt;

use serde::{Deserialize, Serialize};

pub const DID_EXCHANGE_PROTOCOL_URL: &str = "https://didcomm.org/didexchange/1.0";

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum State {
    SendRequest,
    ReceiveRequest,
    SendResponse,
    ReceiveResponse,
    SendComplete,
    ReceiveComplete,
    SendProblemReport,
    ReceiveProblemReport,
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
            "SendRequest" => Ok(State::SendRequest),
            "ReceiveRequest" => Ok(State::ReceiveRequest),
            "SendResponse" => Ok(State::SendResponse),
            "ReceiveResponse" => Ok(State::ReceiveResponse),
            "SendComplete" => Ok(State::SendComplete),
            "ReceiveComplete" => Ok(State::ReceiveComplete),
            "SendProblemReport" => Ok(State::SendProblemReport),
            "ReceiveProblemReport" => Ok(State::ReceiveProblemReport),
            _ => Ok(State::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum UserType {
    Inviter,
    Invitee,
    None,
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
