//! Middleware types.

use std::sync::Arc;

use crate::endpoint::DynEndpoint;
use crate::{Request, Response};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// Middleware that wraps around the remaining middleware chain.
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    /// Asynchronously handle the request, and return a response.
    async fn handle(&self, request: Request, next: Next<'_>) -> crate::Result;

    /// Set the middleware's name. By default it uses the type signature.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

#[async_trait]
impl<F> Middleware for F
where
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(Request, Next<'a>) -> Pin<Box<dyn Future<Output = crate::Result> + 'a + Send>>,
{
    async fn handle(&self, req: Request, next: Next<'_>) -> crate::Result {
        (self)(req, next).await
    }
}

/// The remainder of a middleware chain, including the endpoint.
#[allow(missing_debug_implementations)]
pub struct Next<'a> {
    pub(crate) endpoint: &'a DynEndpoint,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware>],
}

impl Next<'_> {
    /// Asynchronously execute the remaining middleware chain.
    pub async fn run(mut self, req: Request) -> Response {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            match current.handle(req, self).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        } else {
            match self.endpoint.call(req).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        }
    }
}
