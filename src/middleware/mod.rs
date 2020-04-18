//! Middleware types.

use std::sync::Arc;

#[doc(inline)]
pub use http_service::HttpService;
use http_types::Result;

use crate::endpoint::DynEndpoint;
use crate::utils::BoxFuture;
use crate::{Request, Response};

// mod compression;
pub(crate) mod cookies;
mod cors;
// mod default_headers;
mod logger;

// pub use compression::{Compression, Decompression};
pub use cors::{Cors, Origin};
// pub use default_headers::DefaultHeaders;
pub use logger::RequestLogger;

/// Middleware that wraps around remaining middleware chain.
pub trait Middleware<State>: 'static + Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(
        &'a self,
        cx: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Result<Response>>;
}

impl<State, F> Middleware<State> for F
where
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(Request<State>, Next<'a, State>) -> BoxFuture<'a, crate::Result<Response>>,
{
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result<Response>> {
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
    pub fn run(mut self, req: Request<State>) -> BoxFuture<'a, Result<Response>> {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(req, self)
        } else {
            self.endpoint.call(req)
        }
    }
}
