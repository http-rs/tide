use futures::future::{BoxFuture, Future};

use crate::{error::Error, response::IntoResponse, Context, Response};

/// A Tide endpoint.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Tide users.
///
/// In practice, endpoints are functions that take a `Context<State>` as an argument and
/// return a type `T` that implements [`IntoResponse`](crate::response::IntoResponse).
///
/// # Examples
///
/// Endpoints are implemented as asynchronous functions that make use of language features
/// currently only available in Rust Nightly.
///
/// A simple endpoint that is invoked on a `GET` request and returns a `String`:
///
/// ```rust, no_run
/// async fn hello(_cx: tide::Context<()>) -> String {
///     String::from("hello")
/// }
///
/// fn main() {
///     let mut app = tide::App::new();
///     app.at("/hello").get(hello);
///     app.run("127.0.0.1:8000").unwrap()
/// }
/// ```
///
/// An endpoint with similar functionality that does not make use of the `async` keyword would look something like this:
///
/// ```rust, no_run
/// # use core::future::Future;
/// fn hello(_cx: tide::Context<()>) -> impl Future<Output = String> {
///     futures::future::ready(String::from("hello"))
/// }
///
/// fn main() {
///     let mut app = tide::App::new();
///     app.at("/hello").get(hello);
///     app.run("127.0.0.1:8000").unwrap()
/// }
/// ```
///
/// Tide routes will also accept endpoints with `Fn` signatures of this form, but using the `async` keyword has better ergonomics.
pub trait Endpoint<State>: Send + Sync + 'static {
    /// The async result of `call`.
    type Fut: Future<Output = Response> + Send + 'static;

    /// Invoke the endpoint within the given context
    fn call(&self, cx: Context<State>) -> Self::Fut;
}

impl<State, F: Send + Sync + 'static, Fut> Endpoint<State> for F
where
    F: Fn(Context<State>) -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: IntoResponse,
{
    type Fut = BoxFuture<'static, Response>;
    fn call(&self, cx: Context<State>) -> Self::Fut {
        let fut = (self)(cx);
        Box::pin(async move { fut.await.into_response() })
    }
}

/// A convenient `Result` instantiation appropriate for most endpoints.
pub type EndpointResult<T = Response> = Result<T, Error>;
