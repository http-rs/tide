//! Middleware types.

use std::sync::Arc;

use crate::{Endpoint, Request, Response};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// Middleware that wraps around the remaining middleware chain.
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    /// Asynchronously handle the request, and return a response.
    async fn handle(&self, request: Request, next: Next) -> crate::Result;

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
        + Fn(Request, Next) -> Pin<Box<dyn Future<Output = crate::Result> + Send>>,
{
    async fn handle(&self, req: Request, next: Next) -> crate::Result {
        (self)(req, next).await
    }
}

/// The remainder of a middleware chain, including the endpoint.
#[allow(missing_debug_implementations)]
pub struct Next {
    cursor: usize,
    endpoint: Arc<dyn Endpoint>,
    middleware: Arc<Vec<Arc<dyn Middleware>>>,
}

impl Next {
    /// Creates a new Next middleware with an arc to the endpoint and middleware
    pub fn new(endpoint: Arc<dyn Endpoint>, middleware: Arc<Vec<Arc<dyn Middleware>>>) -> Self {
        Self {
            cursor: 0,
            endpoint,
            middleware,
        }
    }

    /// Asynchronously execute the remaining middleware chain.
    pub async fn run(mut self, req: Request) -> Response {
        if let Some(mid) = self.middleware.get(self.cursor) {
            self.cursor += 1;
            match mid.to_owned().handle(req, self).await {
                Ok(response) => response,
                Err(err) => err.into(),
            }
        } else {
            match self.endpoint.call(req).await {
                Ok(response) => response,
                Err(err) => err.into(),
            }
        }
    }
}
