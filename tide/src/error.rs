use core::pin::Pin;
use futures::future::Future;

pub use tide_core::error::{Error, ResponseExt, Result, ResultExt, StringError};

pub(crate) type BoxTryFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'static>>;
