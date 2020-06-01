//! Miscellaneous utilities.

use std::future::Future;
use std::pin::Pin;

use crate::{Middleware, Next, Request};

/// An owned dynamically typed [`Future`] for use in cases where you can't
/// statically type your result or need to add some indirection.
pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Define a middleware that operates on incoming requests.
///
/// This middleware is useful because it is not possible in Rust yet to use
/// closures to define inline middleware.
///
/// # Examples
///
/// ```rust
/// use tide::{utils, Request};
/// use std::time::Instant;
///
/// let mut app = tide::new();
/// app.middleware(utils::Before(|mut request: Request<()>| async move {
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
/// use tide::{utils, http, Response};
///
/// let mut app = tide::new();
/// app.middleware(utils::After(|res: tide::Result| async move {
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
