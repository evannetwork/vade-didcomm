#[cfg(feature = "state_storage")]
use rocksdb::{DBWithThreadMode, MultiThreaded, DB};
use vade::Vade;
use vade_didcomm::VadeDidComm;

#[allow(dead_code)] // usage depends on integration test, so prevent false positives on unused code
const ROCKS_DB_PATH: &str = "./.didcomm_rocks_db";

#[allow(dead_code)] // usage depends on integration test, so prevent false positives on unused code
pub fn read_db(_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    cfg_if::cfg_if! {
        if #[cfg(not(feature = "state_storage"))] {
                return Err(Box::from("read_db cannot be used if 'state_storage' is disabled".to_string()));
        } else {
            let db: DBWithThreadMode<MultiThreaded> = DB::open_default(ROCKS_DB_PATH)?;

            match db.get(_key) {
                Ok(Some(result)) => Ok(String::from_utf8(result)?),
                Ok(None) => Err(format!("{0} not found", _key).into()),
                Err(e) => Err(format!("Error while loading key: {0}, {1}", _key, e).into()),
            }
        }
    }
}

pub async fn get_vade() -> Result<Vade, Box<dyn std::error::Error>> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new()?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}
