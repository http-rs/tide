//! Miscellaneous utilities.

use crate::{Middleware, Next, Request, Response};
pub use async_trait::async_trait;
use std::future::Future;

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
/// app.with(utils::Before(|mut request: Request<()>| async move {
///     request.set_ext(Instant::now());
///     request
/// }));
/// ```
#[derive(Debug)]
pub struct Before<F>(pub F);

#[async_trait]
impl<State, F, Fut> Middleware<State> for Before<F>
where
    State: Clone + Send + Sync + 'static,
    F: Fn(Request<State>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Request<State>> + Send + Sync + 'static,
{
    async fn handle(&self, request: Request<State>, next: Next<'_, State>) -> crate::Result {
        let request = (self.0)(request).await;
        Ok(next.run(request).await)
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
/// app.with(utils::After(|res: Response| async move {
///     match res.status() {
///         http::StatusCode::NotFound => Ok("Page not found".into()),
///         http::StatusCode::InternalServerError => Ok("Something went wrong".into()),
///         _ => Ok(res),
///     }
/// }));
/// ```
///
/// # 404 handling
///
/// When a path cannot be resolved, a 404 error is returned.
/// This response can be caught in the `After` handler, and will have set the `Request<State>` as an extension value.
///
/// Note that you *must* match the `State` type of your request. e.g. if you use `tide::with_state(MyState { .. })`, you must match on `res.ext::<Request<MyState>>()`.
///
/// ```rust
/// use tide::{utils, http, Response, Request};
///
/// let mut app = tide::new();
/// app.with(utils::After(|res: Response| async move {
///     if res.status() == http::StatusCode::NotFound {
///         if let Some(request) = res.ext::<Request<()>>() {
///             tide::log::info!("404 happened on URL {:?}", request.url());
///         }
///     }
///     Ok(res)
/// }));
/// ```
#[derive(Debug)]
pub struct After<F>(pub F);
#[async_trait]
impl<State, F, Fut> Middleware<State> for After<F>
where
    State: Clone + Send + Sync + 'static,
    F: Fn(Response) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = crate::Result> + Send + Sync + 'static,
{
    async fn handle(&self, request: Request<State>, next: Next<'_, State>) -> crate::Result {
        let response = next.run(request).await;
        (self.0)(response).await
    }
}
