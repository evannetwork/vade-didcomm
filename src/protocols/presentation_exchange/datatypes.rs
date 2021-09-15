use serde::{Deserialize, Serialize};
use std::fmt;

pub const PRESENTATION_EXCHANGE_PROTOCOL_URI: &str = "dif/presentation-exchange/definition@v1.0";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresentationExchangeInfo {
    pub r#type: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub presentation_exchange_data: Option<PresentationExchangeData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ProposePresentation {

}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RequestPresentation {

}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Presentation {

}

/// PresentationExchangeData struct is the general structure which contains all optional fields
/// required for all messages of Presentation Exchange protocol.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PresentationExchangeData {
    pub state: State,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub propose_presentation: Option<ProposePresentation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub request_presentation: Option<Vec<RequestPresentation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub presentation: Option<Presentation>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum State {
    SendPresentationRequest,
    ReceivePresentatonRequest,
    SendProposePresentation,
    ReceiveProposePresentation,
    SendPresentation,
    ReceivePresentation,
    Unknown
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
            "SendProposeCredential" => Ok(State::SendPresentation),
            "ReceiveProposeCredential" => Ok(State::ReceivePresentation),
            "SendOfferCredential" => Ok(State::SendProposePresentation),
            "ReceiveOfferCredential" => Ok(State::ReceiveProposePresentation),
            "SendRequestCredential" => Ok(State::SendPresentation),
            "ReceiveRequestCredential" => Ok(State::ReceivePresentation),
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
