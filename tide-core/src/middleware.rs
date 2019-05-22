use crate::{internal::DynEndpoint, Context, Response};
use futures::future::BoxFuture;

use std::sync::Arc;

/// Middleware that wraps around remaining middleware chain.
pub trait Middleware<State>: 'static + Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(&'a self, cx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response>;
}

impl<Data, F> Middleware<Data> for F
where
    F: Send + Sync + 'static + for<'a> Fn(Context<Data>, Next<'a, Data>) -> BoxFuture<'a, Response>,
{
    fn handle<'a>(&'a self, cx: Context<Data>, next: Next<'a, Data>) -> BoxFuture<'a, Response> {
        (self)(cx, next)
    }
}

// TODO (PR258): We don't want this exposed outside of tide repo
/// The remainder of a middleware chain, including the endpoint.
#[allow(missing_debug_implementations)]
pub struct Next<'a, State> {
    pub endpoint: &'a DynEndpoint<State>,
    pub next_middleware: &'a [Arc<dyn Middleware<State>>],
}

impl<'a, State: 'static> Next<'a, State> {
    /// Asynchronously execute the remaining middleware chain.
    pub fn run(mut self, cx: Context<State>) -> BoxFuture<'a, Response> {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(cx, self)
        } else {
            (self.endpoint)(cx)
        }
    }
}
