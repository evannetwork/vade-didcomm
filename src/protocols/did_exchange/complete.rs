use crate::{StepResult, get_step_output_decrypted};

pub fn send_complete(message: &str) -> StepResult {
    return get_step_output_decrypted(
        message,
        "{}",
    );
}

pub fn receive_complete(message: &str) -> StepResult {
    return get_step_output_decrypted(
        message,
        "{}",
    );
}
