use std::{
    convert::TryInto,
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;

use crate::datatypes::{BaseMessage, ExtendedMessage, FromTo};

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

/// Takes an DIDComm message and extracts the basic information that are required.
///
/// # Arguments
/// * `message` - DIDComm message with communication DID document as body
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

    Ok(FromTo {
        from: from_did,
        to: String::from(to_did),
    })
}

/// Adds an id and create_time to stringified DIDComm message.
///
/// # Arguments
/// * `message` - DIDComm message
///
/// # Returns
/// * `string` - stringified message
pub fn fill_message_id_and_timestamps(message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut parsed_message: ExtendedMessage = serde_json::from_str(message)?;

    if parsed_message.id.is_none() {
        parsed_message.id = Some(Uuid::to_string(&Uuid::new_v4()));
    }

    if parsed_message.created_time.is_none() {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)?;
        parsed_message.created_time = Some(since_the_epoch.as_secs());
    }

    Ok(serde_json::to_string(&parsed_message)?)
}
