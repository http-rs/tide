//! For internal use. These APIs will never be stable and
//! are meant to be used internally by the tide repo.

use core::pin::Pin;
use futures::future::{BoxFuture, Future};

use crate::{Context, Response};

/// Convenience alias for pinned box of Future<EndpointResult<T>> + Send + 'static
pub type BoxTryFuture<T> =
    Pin<Box<dyn Future<Output = crate::endpoint::EndpointResult<T>> + Send + 'static>>;

pub type DynEndpoint<State> =
    dyn (Fn(Context<State>) -> BoxFuture<'static, Response>) + 'static + Send + Sync;
