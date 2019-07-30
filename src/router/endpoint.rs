use futures::future::BoxFuture;
use futures::prelude::*;

use crate::middleware::{Response, Context};

/// Tide routes will also accept endpoints with `Fn` signatures of this form, but using the `async` keyword has better ergonomics.
pub trait Endpoint<State>: Send + Sync + 'static {
    /// The async result of `call`.
    type Fut: Future<Output = Response> + Send + 'static;

    /// Invoke the endpoint within the given context
    fn call(&self, cx: Context<State>) -> Self::Fut;
}

impl<State, F: Send + Sync + 'static, Fut> Endpoint<State> for F
where
    F: Fn(Context<State>) -> Fut,
    Fut: Future<Output = Response> + Send + 'static,
{
    type Fut = BoxFuture<'static, Response>;
    fn call(&self, cx: Context<State>) -> Self::Fut {
        let fut = (self)(cx);
        Box::pin(async move { fut.await })
    }
}

/// A convenient `Result` instantiation appropriate for most endpoints.
pub type EndpointResult<T = Response> = Result<T, std::io::Error>;
