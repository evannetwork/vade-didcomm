use rocksdb::{DB, DBWithThreadMode, SingleThreaded};

use crate::utils::SyncResult;

const ROCKS_DB_PATH: &str = "./.didcomm_rocks_db";

pub fn get_db() -> SyncResult<DBWithThreadMode<SingleThreaded>> {
    let db: DBWithThreadMode<SingleThreaded> = DB::open_default(ROCKS_DB_PATH)?;

    return Ok(db);
}

pub fn write_db(key: &str, value: &str) -> SyncResult<()> {
    let db = get_db()?;

    let _ = db.put(key, value);

    return Ok(());
}

pub fn read_db(key: &str) -> SyncResult<String> {
    let db = get_db()?;

    match db.get(key) {
        Ok(Some(result)) => Ok(String::from_utf8(result)?),
        Ok(None) => Err(format!("{0} not found", key).into()),
        Err(e) => Err(format!("Error while loading key: {0}, {1}", key, e).into()),
    }
}

