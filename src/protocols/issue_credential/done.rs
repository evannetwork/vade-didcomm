use crate::protocols::{
    issue_credential::datatypes::Ack,
    protocol::{generate_step_output, StepResult},
};
#[cfg(feature = "state_storage")]
use crate::protocols::issue_credential::{
    credential::{get_current_state, save_state},
    datatypes::{State, UserType}
};

/// Protocol handler for direction: `send`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/ack`
pub fn send_credential_ack(_options: &str, message: &str) -> StepResult {
    let parsed_message: Ack = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let thid = parsed_message
                .thid
                .as_ref()
                .ok_or("Thread id can't be empty")?;

            let current_state: State = get_current_state(thid, &parsed_message.body.user_type)?.parse()?;

            match current_state {
                State::ReceiveIssueCredential => {
                    save_state(thid, &State::Acknowledged, &parsed_message.body.user_type)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "State from {} to {} not allowed",
                        current_state,
                        State::Acknowledged
                    )))
                }
            };
        } else { }
    }

    generate_step_output(&serde_json::to_string(&parsed_message)?, "{}")
}

/// Protocol handler for direction: `receive`, type: `ISSUE_CREDENTIAL_PROTOCOL_URL/ack`
pub fn receive_credential_ack(_options: &str, message: &str) -> StepResult {
    #[allow(unused_variables)] // may not be used afterwards but call is needed to validate input
    let parsed_message: Ack = serde_json::from_str(message)?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "state_storage")] {
            let thid = parsed_message.thid.ok_or("Thread id can't be empty")?;

            if !matches!(&parsed_message.body.user_type, UserType::Holder) {
                return Err(Box::from(
                    "ACK for step 'done' message must be sent from Holder".to_string(),
                ));
            }
            let current_user_type = UserType::Issuer;

            let current_state: State = get_current_state(&thid, &current_user_type)?.parse()?;

            match current_state {
                State::SendIssueCredential => {
                    save_state(&thid, &State::Acknowledged, &parsed_message.body.user_type)?
                }
                _ => {
                    return Err(Box::from(format!(
                        "State from {} to {} not allowed",
                        current_state,
                        State::Acknowledged
                    )))
                }
            };
        } else { }
}

    generate_step_output(message, "{}")
}
