use core::pin::Pin;
use futures::future::Future;

pub use tide_core::error::{EndpointResult, Error, ResponseExt, ResultExt, StringError};

pub(crate) type BoxTryFuture<T> = Pin<Box<dyn Future<Output = EndpointResult<T>> + Send + 'static>>;

#[derive(Debug)]
pub struct StringError(pub String);
impl std::error::Error for StringError {}

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

macro_rules! err_fmt {
    {$($t:tt)*} => {
        crate::error::StringError(format!($($t)*))
    }
}
