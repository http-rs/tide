use async_std::future::Future;
use async_std::sync::Arc;
use async_trait::async_trait;
use http_types::Result;

use crate::middleware::Next;
use crate::{Middleware, Request, Response, State};

/// An HTTP request handler.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Tide users.
///
/// In practice, endpoints are functions that take a `Request, state: State<ServerState>` as an argument and
/// return a type `T` that implements `Into<Response>`.
///
/// # Examples
///
/// Endpoints are implemented as asynchronous functions that make use of language features
/// currently only available in Rust Nightly. For this reason, we have to explicitly enable
/// the attribute will be omitted in most of the documentation.
///
/// A simple endpoint that is invoked on a `GET` request and returns a `String`:
///
/// ```no_run
/// async fn hello(_req: tide::Request, _state: tide::State<()>) -> tide::Result<String> {
///     Ok(String::from("hello"))
/// }
///
/// let mut app = tide::Server::new();
/// app.at("/hello").get(hello);
/// ```
///
/// An endpoint with similar functionality that does not make use of the `async` keyword would look something like this:
///
/// ```no_run
/// # use core::future::Future;
/// fn hello(_req: tide::Request, _state: tide::State<()>) -> impl Future<Output = tide::Result<String>> {
///     async_std::future::ready(Ok(String::from("hello")))
/// }
///
/// let mut app = tide::Server::new();
/// app.at("/hello").get(hello);
/// ```
///
/// Tide routes will also accept endpoints with `Fn` signatures of this form, but using the `async` keyword has better ergonomics.
#[async_trait]
pub trait Endpoint<ServerState: Clone + Send + Sync + 'static>: Send + Sync + 'static {
    /// Invoke the endpoint within the given context
    async fn call(&self, req: Request, state: State<ServerState>) -> crate::Result;
}

pub(crate) type DynEndpoint<ServerState> = dyn Endpoint<ServerState>;

#[async_trait]
impl<ServerState, F, Fut, Res> Endpoint<ServerState> for F
where
    ServerState: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(Request, State<ServerState>) -> Fut,
    Fut: Future<Output = Result<Res>> + Send + 'static,
    Res: Into<Response> + 'static,
{
    async fn call(&self, req: Request, state: State<ServerState>) -> crate::Result {
        let fut = (self)(req, state);
        let res = fut.await?;
        Ok(res.into())
    }
}

pub struct MiddlewareEndpoint<E, ServerState> {
    endpoint: E,
    middleware: Vec<Arc<dyn Middleware<ServerState>>>,
}

impl<E: Clone, ServerState> Clone for MiddlewareEndpoint<E, ServerState> {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

impl<E, ServerState> std::fmt::Debug for MiddlewareEndpoint<E, ServerState> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "MiddlewareEndpoint (length: {})",
            self.middleware.len(),
        )
    }
}

impl<E, ServerState> MiddlewareEndpoint<E, ServerState>
where
    ServerState: Clone + Send + Sync + 'static,
    E: Endpoint<ServerState>,
{
    pub fn wrap_with_middleware(ep: E, middleware: &[Arc<dyn Middleware<ServerState>>]) -> Self {
        Self {
            endpoint: ep,
            middleware: middleware.to_vec(),
        }
    }
}

#[async_trait]
impl<E, ServerState> Endpoint<ServerState> for MiddlewareEndpoint<E, ServerState>
where
    ServerState: Clone + Send + Sync + 'static,
    E: Endpoint<ServerState>,
{
    async fn call(&self, req: Request, state: State<ServerState>) -> crate::Result {
        let next = Next {
            endpoint: &self.endpoint,
            next_middleware: &self.middleware,
        };
        Ok(next.run(req, state).await)
    }
}

#[async_trait]
impl<ServerState: Clone + Send + Sync + 'static> Endpoint<ServerState>
    for Box<dyn Endpoint<ServerState>>
{
    async fn call(&self, req: Request, state: State<ServerState>) -> crate::Result {
        self.as_ref().call(req, state).await
    }
}
