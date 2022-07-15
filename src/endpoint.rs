use async_std::future::Future;
use async_std::sync::Arc;
use async_trait::async_trait;
use http_types::Result;

use crate::middleware::Next;
use crate::{Middleware, Request, Response};

/// An HTTP request handler.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Tide users.
///
/// In practice, endpoints are functions that take a `Request<State>` as an argument and
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
/// async fn hello(_req: tide::Request) -> tide::Result<String> {
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
/// fn hello(_req: tide::Request) -> impl Future<Output = tide::Result<String>> {
///     async_std::future::ready(Ok(String::from("hello")))
/// }
///
/// let mut app = tide::Server::new();
/// app.at("/hello").get(hello);
/// ```
///
/// Tide routes will also accept endpoints with `Fn` signatures of this form, but using the `async` keyword has better ergonomics.
#[async_trait]
pub trait Endpoint: Send + Sync + 'static {
    /// Invoke the endpoint within the given context
    async fn call(&self, req: Request) -> crate::Result;
}

pub(crate) type DynEndpoint = dyn Endpoint;

#[async_trait]
impl<F, Fut, Res> Endpoint for F
where
    F: Send + Sync + 'static + Fn(Request) -> Fut,
    Fut: Future<Output = Result<Res>> + Send + 'static,
    Res: Into<Response> + 'static,
{
    async fn call(&self, req: Request) -> crate::Result {
        let fut = (self)(req);
        let res = fut.await?;
        Ok(res.into())
    }
}

pub(crate) struct MiddlewareEndpoint<E, State> {
    endpoint: E,
    middleware: Vec<Arc<dyn Middleware<State>>>,
}

impl<E: Clone, State> Clone for MiddlewareEndpoint<E, State> {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

impl<E, State> std::fmt::Debug for MiddlewareEndpoint<E, State> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "MiddlewareEndpoint (length: {})",
            self.middleware.len(),
        )
    }
}

impl<E, State> MiddlewareEndpoint<E, State>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint,
{
    pub(crate) fn wrap_with_middleware(
        ep: E,
        middleware: &[Arc<dyn Middleware<State>>],
    ) -> Box<dyn Endpoint + Send + Sync + 'static> {
        if middleware.is_empty() {
            Box::new(ep)
        } else {
            Box::new(Self {
                endpoint: ep,
                middleware: middleware.to_vec(),
            })
        }
    }
}

#[async_trait]
impl<E, State> Endpoint for MiddlewareEndpoint<E, State>
where
    State: Clone + Send + Sync + 'static,
    E: Endpoint,
{
    async fn call(&self, req: Request) -> crate::Result {
        let next = Next {
            endpoint: &self.endpoint,
            next_middleware: &self.middleware,
        };
        Ok(next.run(req).await)
    }
}

#[async_trait]
impl Endpoint for Box<dyn Endpoint> {
    async fn call(&self, request: Request) -> crate::Result {
        self.as_ref().call(request).await
    }
}
