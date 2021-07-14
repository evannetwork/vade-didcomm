use rocksdb::{DBWithThreadMode, SingleThreaded, DB};

use crate::utils::SyncResult;

const ROCKS_DB_PATH: &str = "./.didcomm_rocks_db";

/// Return a new instance of the rocks db.
fn get_db() -> SyncResult<DBWithThreadMode<SingleThreaded>> {
    let db: DBWithThreadMode<SingleThreaded> = DB::open_default(ROCKS_DB_PATH)?;

    return Ok(db);
}

/// Write a value into the rocks db.
///
/// # Arguments
/// * `key` - key to save the value for
/// * `value` - string value to store
pub fn write_db(key: &str, value: &str) -> SyncResult<()> {
    let db = get_db()?;

    let _ = db.put(key, value);

    return Ok(());
}

/// Gets a value from the rocks db.
///
/// # Arguments
/// * `key` - key to load the value for
///
/// # Returns
/// * `String` - stored value
pub fn read_db(key: &str) -> SyncResult<String> {
    let db = get_db()?;

    match db.get(key) {
        Ok(Some(result)) => Ok(String::from_utf8(result)?),
        Ok(None) => Err(format!("{0} not found", key).into()),
        Err(e) => Err(format!("Error while loading key: {0}, {1}", key, e).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_use_rocks_db() -> SyncResult<()> {
        let _ = write_db("test1", "helloooo");
        let result = read_db("test1")?;

        assert_eq!(result, "helloooo");

        return Ok(());
    }
}
