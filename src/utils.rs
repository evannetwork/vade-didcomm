use std::convert::TryInto;

use crate::datatypes::{BaseMessage, FromTo};

/// Formats an vector into an array dynamically.
///
/// # Arguments
/// * `v` - vector to format
///
/// # Returns
/// * `Array` - the transformed array
pub fn vec_to_array<T, const N: usize>(v: Vec<T>) -> Result<[T; N], Box<dyn std::error::Error>> {
    v.try_into()
        .map_err(|_e| Box::from("could not format vec to array"))
}

/// Takes an didcomm message and extracts the basic information that are required.
///
/// # Arguments
/// * `message` - didcomm message with communication DID document as body
///
/// # Returns
/// * `ExchangeInfo` - necessary information
pub fn get_from_to_from_message(
    message: BaseMessage,
) -> Result<FromTo, Box<dyn std::error::Error>> {
    let from_did = message.from.ok_or("from is required")?;

    let to_vec = message.to.ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from(
            "DID exchange requires at least one did in the to field.",
        ));
    }
    let to_did = &to_vec[0];

    return Ok(FromTo {
        from: String::from(from_did),
        to: String::from(to_did),
    });
}
