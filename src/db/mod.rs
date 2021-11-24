cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod local_storage;
        pub(crate) use local_storage::*;
    } else if #[cfg(feature = "debug_db")] {
        mod debug;
        pub use debug::*;
    } else {
        mod rocks_db;
        pub(crate) use rocks_db::*;
    }
}
