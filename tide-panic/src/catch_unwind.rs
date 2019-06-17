use futures::future::{BoxFuture, FutureExt, TryFutureExt};
use http::status::StatusCode;
use std::{
    any::Any,
    panic::{AssertUnwindSafe, RefUnwindSafe},
};
use tide_core::{
    middleware::{Middleware, Next},
    response::IntoResponse,
    Context, Response,
};

/// A [`Middleware`] that will catch any panics from later middleware or handlers and return a
/// response to the client.
///
/// It is **not** recommended to use this middleware for a general try/catch mechanism. The
/// [`Result`] type is more appropriate to use for middleware/handlers that can fail on a regular
/// basis.  Additionally, this middleware is not guaranteed to catch all panics, see the "Notes"
/// section in the [`std::panic::catch_unwind`] docs.
pub struct CatchUnwind {
    f: Box<dyn Fn(Box<dyn Any + Send + 'static>) -> Response + Send + Sync>,
}

impl CatchUnwind {
    /// Create a [`CatchUnwind`] which will respond with [`StatusCode::INTERNAL_SERVER_ERROR`] when
    /// any panic is caught.
    pub fn new() -> Self {
        Self::with_response(|_| {
            "Internal server error"
                .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                .into_response()
        })
    }

    /// Create a [`CatchUnwind`] with a custom function to generate the response, the function will
    /// be passed the caught panic.
    pub fn with_response(
        response: impl Fn(Box<dyn Any + Send + 'static>) -> Response + Send + Sync + 'static,
    ) -> Self {
        Self {
            f: Box::new(response),
        }
    }
}

impl<State: RefUnwindSafe + 'static> Middleware<State> for CatchUnwind {
    fn handle<'a>(&'a self, cx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        AssertUnwindSafe(next.run(cx))
            .catch_unwind()
            .unwrap_or_else(move |err| (self.f)(err))
            .boxed()
    }
}

impl Default for CatchUnwind {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CatchUnwind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CatchUnwind").finish()
    }
}
