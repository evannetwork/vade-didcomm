use crate::{
    datatypes::{Ack, BaseMessage, ExtendedMessage},
    get_from_to_from_message,
    presentation::get_presentation,
    protocols::protocol::{generate_step_output, StepResult},
};

/// Protocol handler for direction: `send`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn send_presentation_ack(message: &str) -> StepResult {
    let parsed_message: ExtendedMessage = serde_json::from_str(message)?;
    let data = &serde_json::to_string(
        &parsed_message
            .body
            .ok_or("Presentation data not provided.")?,
    )?;
    let ack: Ack = serde_json::from_str(&data)?;

    let base_message: BaseMessage = BaseMessage {
        from: parsed_message.from,
        r#type: parsed_message.r#type,
        to: Some(parsed_message.to.ok_or("To DID not provided.")?.to_vec()),
    };
    let exchange_info = get_from_to_from_message(base_message)?;
    let thid = parsed_message
        .thid
        .to_owned()
        .ok_or("Thread id can't be empty")?;

    let saved_presentation = get_presentation(&exchange_info.from, &exchange_info.to, &thid)?;
    if saved_presentation.presentation_attach.is_none() {
        panic!("No request for presentation found.");
    }

    let presentation_attach = saved_presentation
        .presentation_attach
        .ok_or("No attached presentation")?;

    let presentation_data = presentation_attach
        .get(0)
        .ok_or("No data found for attached Presentation")?;
    
    if !presentation_data.r#type.contains("presentation") {
        panic!("Cant send ack without receiving presentation.");
    }

    generate_step_output(&serde_json::to_string(&ack)?, "{}")
}
/// Protocol handler for direction: `receive`, type: `PRESENT_PROOF_PROTOCOL_URL/ack`
pub fn receive_presentation_ack(message: &str) -> StepResult {
    generate_step_output(message, "{}")
}
