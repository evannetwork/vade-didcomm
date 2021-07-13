use std::convert::TryInto;

pub type AsyncResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub trait ResultAsyncifier<T> {
    fn asyncify(self) -> AsyncResult<T>;
}
impl<T> ResultAsyncifier<T> for Result<T, Box<dyn std::error::Error + Send + Sync>> {
    fn asyncify(self) -> AsyncResult<T> {
        self.map_err(|err| err.to_string().into())
    }
}

pub type SyncResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}
