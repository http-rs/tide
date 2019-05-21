//! Module to export tide_core errors

use core::pin::Pin;
use futures::future::Future;

pub use tide_core::error::{
    EndpointResult, Error, ResponseExt, ResultDynErrExt, ResultExt, StringError,
};

pub(crate) type BoxTryFuture<T> = Pin<Box<dyn Future<Output = EndpointResult<T>> + Send + 'static>>;
