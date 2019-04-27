use futures::future::BoxFuture;
use std::sync::Arc;

use crate::{endpoint::DynEndpoint, Context, Response};

mod cookies;
mod default_headers;
mod logger;

pub use self::{cookies::CookiesMiddleware, default_headers::DefaultHeaders, logger::RootLogger};

/// Middleware that wraps around remaining middleware chain.
pub trait Middleware<AppData>: 'static + Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(
        &'a self,
        cx: Context<AppData>,
        next: Next<'a, AppData>,
    ) -> BoxFuture<'a, Response>;
}

impl<Data, F> Middleware<Data> for F
where
    F: Send + Sync + 'static + for<'a> Fn(Context<Data>, Next<'a, Data>) -> BoxFuture<'a, Response>,
{
    fn handle<'a>(&'a self, cx: Context<Data>, next: Next<'a, Data>) -> BoxFuture<'a, Response> {
        (self)(cx, next)
    }
}

/// The remainder of a middleware chain, including the endpoint.
pub struct Next<'a, AppData> {
    pub(crate) endpoint: &'a DynEndpoint<AppData>,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware<AppData>>],
}

impl<'a, AppData: 'static> Next<'a, AppData> {
    /// Asynchronously execute the remaining middleware chain.
    pub fn run(mut self, cx: Context<AppData>) -> BoxFuture<'a, Response> {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(cx, self)
        } else {
            (self.endpoint)(cx)
        }
    }
}
