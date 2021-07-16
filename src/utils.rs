use std::convert::TryInto;

use crate::datatypes::{BaseMessage, FromTo};

pub type AsyncResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub trait ResultAsyncifier<T> {
    fn asyncify(self) -> AsyncResult<T>;
}
impl<T> ResultAsyncifier<T> for Result<T, Box<dyn std::error::Error + Send + Sync>> {
    fn asyncify(self) -> AsyncResult<T> {
        self.map_err(|err| err.to_string().into())
    }
}

pub type SyncResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Formats an vector into an array dynamically.
///
/// # Arguments
/// * `v` - vector to format
///
/// # Returns
/// * `Array` - the transformed array
pub fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

/// Takes an didcomm message and extracts the basic information that are required.
///
/// # Arguments
/// * `message` - didcomm message with communication did document as body
///
/// # Returns
/// * `ExchangeInfo` - necessary information
pub fn get_from_to_from_message(message: BaseMessage) -> SyncResult<FromTo> {
    let from_did = message.from.ok_or("from is required")?;

    let to_vec = message.to.ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from(
            "did exchange requires at least one did in the to field.",
        ));
    }
    let to_did = &to_vec[0];

    return Ok(FromTo {
        from: String::from(from_did),
        to: String::from(to_did),
    });
}
