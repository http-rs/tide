//! Miscellaneous utilities.

use crate::{Middleware, Next, Request, Response, State};
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
/// app.with(utils::Before(|mut req: Request, state: tide::State<()>| async move {
///     req.set_ext(Instant::now());
///     (req, state)
/// }));
/// ```
#[derive(Debug)]
pub struct Before<F>(pub F);

#[async_trait]
impl<ServerState, F, Fut> Middleware<ServerState> for Before<F>
where
    ServerState: Clone + Send + Sync + 'static,
    F: Fn(Request, State<ServerState>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = (Request, State<ServerState>)> + Send + Sync + 'static,
{
    async fn handle(
        &self,
        req: Request,
        state: State<ServerState>,
        next: Next<'_, ServerState>,
    ) -> crate::Result {
        let (req, state) = (self.0)(req, state).await;
        Ok(next.run(req, state).await)
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
#[derive(Debug)]
pub struct After<F>(pub F);
#[async_trait]
impl<ServerState, F, Fut> Middleware<ServerState> for After<F>
where
    ServerState: Clone + Send + Sync + 'static,
    F: Fn(Response) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = crate::Result> + Send + Sync + 'static,
{
    async fn handle(
        &self,
        req: Request,
        state: State<ServerState>,
        next: Next<'_, ServerState>,
    ) -> crate::Result {
        let res = next.run(req, state).await;
        (self.0)(res).await
    }
}
