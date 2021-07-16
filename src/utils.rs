use std::{
    convert::TryInto,
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;

use crate::datatypes::{BaseMessage, ExtendedMessage, FromTo};

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
pub fn vec_to_array<T, const N: usize>(v: Vec<T>) -> SyncResult<[T; N]> {
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
pub fn get_from_to_from_message(message: BaseMessage) -> SyncResult<FromTo> {
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

/// Adds an id and create_time to stringified DIDComm message.
///
/// # Arguments
/// * `message` - DIDComm message
///
/// # Returns
/// * `string` - stringified message
pub fn fill_message_id_and_timestamps(message: &str) -> SyncResult<String> {
    let mut parsed_message: ExtendedMessage = serde_json::from_str(message)?;

    if !parsed_message.id.is_some() {
        parsed_message.id = Some(Uuid::to_string(&Uuid::new_v4()));
    }

    if !parsed_message.created_time.is_some() {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)?;
        parsed_message.created_time = Some(since_the_epoch.as_secs());
    }

    return Ok(serde_json::to_string(&parsed_message)?);
}
