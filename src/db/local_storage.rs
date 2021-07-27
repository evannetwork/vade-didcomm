use web_sys::Storage;

const LOCAL_STORAGE_PREFIX: &str = "equs-evan-didcomm-db";

pub fn get_storage() -> Result<Storage, Box<dyn std::error::Error>> {
    let window = web_sys::window().ok_or_else(|| "could not get window".to_string())?;
    if let Ok(Some(local_storage)) = window.local_storage() {
        Ok(local_storage)
    } else {
        Err(Box::from("could not get local storage"))
    }
}

/// Write a value into local storage.
///
/// # Arguments
/// * `key` - key to save the value for
/// * `value` - string value to store
pub fn write_db(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    get_storage()?
        .set_item(&format!("{}:{}", LOCAL_STORAGE_PREFIX, key), value)
        .map_err(|err| {
            Box::from(
                err.as_string()
                    .unwrap_or_else(|| "could not write to local storage".to_string()),
            )
        })
}

/// Gets a value from local storage.
///
/// # Arguments
/// * `key` - key to load the value for
///
/// # Returns
/// * `String` - stored value
pub fn read_db(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    get_storage()?
        .get_item(&format!("{}:{}", LOCAL_STORAGE_PREFIX, key))
        .map_err(|err| {
            err.as_string()
                .unwrap_or_else(|| "could read from local storage".to_string())
        })?
        .ok_or_else(|| Box::from("".to_string()))
}
