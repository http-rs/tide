//! Middleware types.

#[doc(inline)]
pub use crate::http_service::{Body, HttpService, Request, Response};
pub use context::Context;

use crate::Exception;
use futures::future::BoxFuture;
use std::sync::Arc;

/// Middleware that wraps around remaining middleware chain.
pub trait Middleware<State: Send + Sync + 'static>: 'static + Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(
        &'a self,
        req: Request,
        client: State,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Result<Response, Exception>>;
}

// This allows functions to work as middleware too.
impl<F, State: Send + Sync + 'static> Middleware<State> for F
where
    F: Send
        + Sync
        + 'static
        + for<'a> Fn(Request, State, Next<'a, State>) -> BoxFuture<'a, Result<Response, Exception>>,
{
    fn handle<'a>(
        &'a self,
        req: Request,
        state: State,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Result<Response, Exception>> {
        (self)(req, state, next)
    }
}

/// The remainder of a middleware chain, including the endpoint.
#[allow(missing_debug_implementations)]
pub struct Next<'a, State: Send + Sync + 'static> {
    next_middleware: &'a [Arc<dyn Middleware<State>>],
    endpoint: &'a (dyn (Fn(Request, State) -> BoxFuture<'static, Result<Response, Exception>>)
             + 'static
             + Send
             + Sync),
}

impl<'a, State: Send + Sync + 'static> Next<'a, State> {
    /// Create a new instance
    pub fn new(
        next: &'a [Arc<dyn Middleware<State>>],
        endpoint: &'a (dyn (Fn(Request, State) -> BoxFuture<'static, Result<Response, Exception>>)
                 + 'static
                 + Send
                 + Sync),
    ) -> Self {
        Self {
            endpoint,
            next_middleware: next,
        }
    }

    /// Asynchronously execute the remaining middleware chain.
    pub fn run(mut self, req: Request, state: State) -> BoxFuture<'a, Result<Response, Exception>> {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(req, self, state)
        } else {
            (self.endpoint)(req, state)
        }
    }
}

mod context {
    use route_recognizer::Params;
    use std::sync::Arc;

    /// State associated with a request-response lifecycle.
    ///
    /// The `Context` gives endpoints access to basic information about the incoming
    /// request, route parameters, and various ways of accessing the request's body.
    ///
    /// Contexts also provide *extensions*, a type map primarily used for low-level
    /// communication between middleware and endpoints.
    #[derive(Debug)]
    pub struct Context<State> {
        state: Arc<State>,
        route_params: Params,
    }

    impl<State> Context<State> {
        /// Create a new instance
        pub fn new(state: Arc<State>, route_params: Params) -> Self {
            Self {
                state,
                route_params,
            }
        }
    }
}
