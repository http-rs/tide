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

    /// Set the middleware's name. By default it uses the type signature.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// Define a middleware that operates on incoming requests.
///
/// This middleware is useful because it is not possible in Rust yet to use
/// closures to define inline middleware.
///
/// # Examples
///
/// ```rust
/// use tide::{Before, Request};
/// use std::time::Instant;
///
/// let mut app = tide::new();
/// app.middleware(Before(|mut request: Request<()>| async move {
///     request.set_ext(Instant::now());
///     request
/// }));
/// ```
#[derive(Debug)]
pub struct Before<F>(pub F);
impl<State, F, Fut> Middleware<State> for Before<F>
where
    State: Send + Sync + 'static,
    F: Fn(Request<State>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Request<State>> + Send + Sync,
{
    fn handle<'a>(
        &'a self,
        request: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result> {
        Box::pin(async move {
            let request = (self.0)(request).await;
            next.run(request).await
        })
    }
}

/// Define a middleware that operates on outgoing responses.
///
/// This middleware is useful because it is not possible in Rust yet to use
/// closures to define inline middleware.
///
/// # Examples
///
/// ```rust
/// use tide::{After, Response, http};
///
/// let mut app = tide::new();
/// app.middleware(After(|res: tide::Result| async move {
///     let res = res.unwrap_or_else(|e| Response::new(e.status()));
///     match res.status() {
///         http::StatusCode::NotFound => Ok("Page not found".into()),
///         http::StatusCode::InternalServerError => Ok("Something went wrong".into()),
///         _ => Ok(res),
///     }
/// }));
/// ```
#[derive(Debug)]
pub struct After<F>(pub F);
impl<State, F, Fut> Middleware<State> for After<F>
where
    State: Send + Sync + 'static,
    F: Fn(crate::Result) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = crate::Result> + Send + Sync,
{
    fn handle<'a>(
        &'a self,
        request: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result> {
        Box::pin(async move {
            let result = next.run(request).await;
            (self.0)(result).await
        })
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
