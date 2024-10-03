use std::error::Error;

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;
pub type Result<T = (), Error = BoxError> = std::result::Result<T, Error>;
