use rocksdb::{DBWithThreadMode, IteratorMode, MultiThreaded, DB};

const ROCKS_DB_PATH: &str = "./.didcomm_rocks_db";

/// Return a new instance of the rocks db.
fn get_db() -> Result<DBWithThreadMode<MultiThreaded>, Box<dyn std::error::Error>> {
    let db: DBWithThreadMode<MultiThreaded> = DB::open_default(ROCKS_DB_PATH)?;

    Ok(db)
}

/// Write a value into the rocks db.
///
/// # Arguments
/// * `key` - key to save the value for
/// * `value` - string value to store
pub fn write_db(key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db = get_db()?;

    db.put(key, value)?;

    Ok(())
}

/// Gets a value from the rocks db.
///
/// # Arguments
/// * `key` - key to load the value for
///
/// # Returns
/// * `String` - stored value
pub fn read_db(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let db = get_db()?;

    match db.get(key) {
        Ok(Some(result)) => Ok(String::from_utf8(result)?),
        Ok(None) => Err(format!("{0} not found", key).into()),
        Err(e) => Err(format!("Error while loading key: {0}, {1}", key, e).into()),
    }
}

/// Gets a list of values matching with key prefix from the rocks db.
///
/// # Arguments
/// * `prefix` - key prefix to match values for
///
/// # Returns
/// * `Vec<String>` - stored values
pub fn search_db_keys(prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut values: Vec<String> = Vec::new();
    let db = get_db()?;
    let mode = IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward);

    let result = db
        .iterator(mode)
        .take_while(|(k, _)| k.starts_with(prefix.as_bytes()));
    for (_, val) in result {
        let value = String::from_utf8((*val).to_vec())?;
        values.push(value);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "state_storage")]
    fn can_use_rocks_db() -> Result<(), Box<dyn std::error::Error>> {
        write_db("test1", "helloooo")?;
        let result = read_db("test1")?;

        assert_eq!(result, "helloooo");

        Ok(())
    }
}
