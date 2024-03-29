cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod local_storage;
        pub(crate) use local_storage::*;
    } else if #[cfg(feature = "debug_db")] {
        mod debug;
        pub use debug::*;
    } else {
        #[cfg(feature = "state_storage")]
        mod rocks_db;
        #[cfg(feature = "state_storage")]
        pub(crate) use rocks_db::*;
    }
}
