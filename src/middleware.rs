//! Middleware types.

use std::sync::Arc;

use crate::endpoint::DynEndpoint;
use crate::utils::BoxFuture;
use crate::Request;

// mod compression;
// mod default_headers;

// pub use compression::{Compression, Decompression};
// pub use default_headers::DefaultHeaders;

/// Middleware that wraps around the remaining middleware chain.
pub trait Middleware<State>: 'static + Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(
        &'a self,
        request: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result>;

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

impl<State, F> Middleware<State> for F
where
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(Request<State>, Next<'a, State>) -> BoxFuture<'a, crate::Result>,
{
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result> {
        (self)(req, next)
    }
}

/// The remainder of a middleware chain, including the endpoint.
#[allow(missing_debug_implementations)]
pub struct Next<'a, State> {
    pub(crate) endpoint: &'a DynEndpoint<State>,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware<State>>],
}

impl<'a, State: 'static> Next<'a, State> {
    /// Asynchronously execute the remaining middleware chain.
    #[must_use]
    pub fn run(mut self, req: Request<State>) -> BoxFuture<'a, crate::Result> {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(req, self)
        } else {
            self.endpoint.call(req)
        }
    }
}
