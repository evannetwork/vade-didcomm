use std::{
    convert::TryInto,
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;

use crate::{
    datatypes::{BaseMessage, ExtendedMessage, FromTo},
    db::write_db,
};

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
    message: &BaseMessage,
) -> Result<FromTo, Box<dyn std::error::Error>> {
    let from_did = message.from.as_ref().ok_or("from is required")?;

    let to_vec = message.to.as_ref().ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from(
            "DID exchange requires at least one did in the to field.",
        ));
    }
    let to_did = &to_vec[0];

    Ok(FromTo {
        from: from_did.to_owned(),
        to: String::from(to_did),
    })
}

/// Write a didcomm_send/didcomm_receive raw message to rocks_db
///
/// # Arguments
/// * `message` - Raw message
pub fn write_raw_message_to_db(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_raw_message: ExtendedMessage = serde_json::from_str(message)?;
    
    write_db(
        &format!(
            "message_{}_{}",
            parsed_raw_message.thid.unwrap_or(parsed_raw_message.id.clone().ok_or("id is missing")?),
            parsed_raw_message.id.ok_or("id is missing")?
        ),
        message,
    )
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

pub(crate) mod hex_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &Option<[u8; 32]>, s: S) -> Result<S::Ok, S::Error> {
        let hex_string = v.as_ref().map(hex::encode);
        <Option<String>>::serialize(&hex_string, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<[u8; 32]>, D::Error> {
        let hex_string = <Option<String>>::deserialize(d)?;
        match hex_string {
            Some(v) => {
                let hex_decoded = hex::decode(v).map_err(serde::de::Error::custom)?;
                let mut arr: [u8; 32] = Default::default();
                arr.copy_from_slice(&hex_decoded[..32]);
                Ok(Some(arr))
            }
            None => Ok(None),
        }
    }
}
