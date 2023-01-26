use crate::{
    datatypes::ExtendedMessage,
    protocols::{
        did_exchange::DID_EXCHANGE_PROTOCOL_URL,
        protocol::{generate_step_output, StepResult},
    },
};
#[cfg(feature = "state_storage")]
use crate::protocols::{
    did_exchange::datatypes::{State, UserType},
    did_exchange::did_exchange::{get_current_state, save_state},
};

/// protocol handler for direction: `send`, type: `DID_EXCHANGE_PROTOCOL_URL/complete`
/// just ensures to set the correct message type, before the message will be sent (first time for
/// DID exchange, that a encrypted message will be sent)
pub fn send_complete(_options: &str, message: &str) -> StepResult {
    let mut parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    parsed_message.r#type = format!("{}/complete", DID_EXCHANGE_PROTOCOL_URL);

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let thid = parsed_message.thid.as_ref().ok_or("Thread id can't be empty")?;
            let current_state: State = get_current_state(&thid, &UserType::Inviter)?.parse()?;

            match current_state {
                State::ReceiveResponse => {
                    save_state(&thid, &State::SendComplete, &UserType::Inviter)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "State from {} to {} not allowed",
                        current_state,
                        State::SendComplete
                    )))
                }
            };
        } else { }
    }

    generate_step_output(&serde_json::to_string(&parsed_message)?, "{}")
}

/// protocol handler for direction: `receive`, type: `DID_EXCHANGE_PROTOCOL_URL/complete`
pub fn receive_complete(_options: &str, message: &str) -> StepResult {
    #[allow(unused_variables)] // may not be used afterwards but call is needed to validate input
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {

            let thid = parsed_message.thid.as_ref().ok_or("Thread id can't be empty")?;
            let current_state: State = get_current_state(&thid, &UserType::Invitee)?.parse()?;

            match current_state {
                State::SendResponse => save_state(&thid, &State::ReceiveComplete, &UserType::Invitee)?,
                _ => {
                    return Err(Box::from(format!(
                        "State from {} to {} not allowed",
                        current_state,
                        State::ReceiveComplete
                    )))
                }
            };
        } else { }
    }

    generate_step_output(message, "{}")
}
