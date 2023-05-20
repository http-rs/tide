use async_std::future::Future;
use async_trait::async_trait;
use http_types::Result;

use crate::{Request, Response};

/// An HTTP request handler.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Tide users.
///
/// In practice, endpoints are functions that take a `Request` as an argument and
/// return a type `T` that implements `Into<Response>`.
///
/// # Examples
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

#[async_trait]
impl Endpoint for Box<dyn Endpoint> {
    async fn call(&self, request: Request) -> crate::Result {
        self.as_ref().call(request).await
    }
}
