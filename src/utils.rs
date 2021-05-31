pub type AsyncResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub trait ResultAsyncifier<T> {
    fn asyncify(self) -> AsyncResult<T>;
}
impl<T> ResultAsyncifier<T> for Result<T, Box<dyn std::error::Error + Send + Sync>> {
    fn asyncify(self) -> AsyncResult<T> {
        self.map_err(|err| err.to_string().into())
    }
}
