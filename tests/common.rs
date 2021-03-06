use rocksdb::{DBWithThreadMode, MultiThreaded, DB};
use vade::Vade;
use vade_didcomm::VadeDidComm;

#[allow(dead_code)] // usage depends on integration test, so prevent false positives on unused code
const ROCKS_DB_PATH: &str = "./.didcomm_rocks_db";

#[allow(dead_code)] // usage depends on integration test, so prevent false positives on unused code
pub fn read_db(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let db: DBWithThreadMode<MultiThreaded> = DB::open_default(ROCKS_DB_PATH)?;

    match db.get(key) {
        Ok(Some(result)) => Ok(String::from_utf8(result)?),
        Ok(None) => Err(format!("{0} not found", key).into()),
        Err(e) => Err(format!("Error while loading key: {0}, {1}", key, e).into()),
    }
}

pub async fn get_vade() -> Result<Vade, Box<dyn std::error::Error>> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new()?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}
