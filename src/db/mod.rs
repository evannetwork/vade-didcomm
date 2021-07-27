#[cfg(target_arch = "wasm32")]
mod local_storage;
#[cfg(not(target_arch = "wasm32"))]
mod rocks_db;

#[cfg(target_arch = "wasm32")]
pub(crate) use local_storage::*;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use rocks_db::*;
