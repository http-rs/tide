//! Middleware types.

use std::sync::Arc;

use crate::endpoint::DynEndpoint;
use crate::{Request, Response, State};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;

/// Middleware that wraps around the remaining middleware chain.
#[async_trait]
pub trait Middleware<ServerState>: Send + Sync + 'static {
    /// Asynchronously handle the request, and return a response.
    async fn handle(
        &self,
        request: Request,
        state: State<ServerState>,
        next: Next<'_, ServerState>,
    ) -> crate::Result;

    /// Set the middleware's name. By default it uses the type signature.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

#[async_trait]
impl<ServerState, F> Middleware<ServerState> for F
where
    ServerState: Clone + Send + Sync + 'static,
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(
            Request,
            State<ServerState>,
            Next<'a, ServerState>,
        ) -> Pin<Box<dyn Future<Output = crate::Result> + 'a + Send>>,
{
    async fn handle(
        &self,
        req: Request,
        state: State<ServerState>,
        next: Next<'_, ServerState>,
    ) -> crate::Result {
        (self)(req, state, next).await
    }
}

/// The remainder of a middleware chain, including the endpoint.
#[allow(missing_debug_implementations)]
pub struct Next<'a, ServerState> {
    pub(crate) endpoint: &'a DynEndpoint<ServerState>,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware<ServerState>>],
}

impl<ServerState> Next<'_, ServerState>
where
    ServerState: Clone + Send + Sync + 'static,
{
    /// Asynchronously execute the remaining middleware chain.
    pub async fn run(mut self, req: Request, state: State<ServerState>) -> Response {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            match current.handle(req, state, self).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        } else {
            match self.endpoint.call(req, state).await {
                Ok(request) => request,
                Err(err) => err.into(),
            }
        }
    }
}
