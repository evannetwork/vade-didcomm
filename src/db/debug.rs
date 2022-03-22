//! Simple database for debug purposes.
//!
//! Only intended for debugging as it does allow to inspect written data.
//!
//! Does NOT ensure thread safety, therefore parallel calls may overwrite the "database" file.

use std::fs;
use std::collections::HashMap;
use serde_json::json;

const DEBUG_DB_PATH: &str = "./.didcomm_debug_db.json";

fn get_storage() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let json_text: String;
    if !std::path::Path::new(DEBUG_DB_PATH).exists() {
        fs::write(DEBUG_DB_PATH, "{}")?;
        json_text = "{}".to_string();
    } else {
        json_text = fs::read_to_string(DEBUG_DB_PATH)?;
    }
    serde_json::from_str(&json_text).map_err(|e| Box::from(e.to_string()))
}

pub fn write_db(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut storage = get_storage()?;
    storage[key] = json!(value);
    fs::write(DEBUG_DB_PATH, serde_json::to_string_pretty(&storage)?)?;

    Ok(())
}

pub fn read_db(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let storage = get_storage()?;
    storage[key]
        .as_str()
        .ok_or(format!("key {} not found in debug db", key))
        .map(|v| v.to_string())
        .map_err(|e| Box::from(e.to_string()))
}

/// Gets a list of values matching with key prefix from local file.
///
/// # Arguments
/// * `prefix` - key prefix to match values for
///
/// # Returns
/// * `Vec<String>` - stored values
pub fn search_db_keys(prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut values: Vec<String> = Vec::new();
    let storage = &serde_json::to_string(&get_storage()?)?;
    let storage_map: HashMap<&str, &str> = serde_json::from_str(storage)?;

    for (key, value) in storage_map {
        if key.contains(prefix) {
            values.push(value.to_string());
        }
    }

    return Ok(values);
}
