use futures::future::{BoxFuture, Future};

use crate::{response::IntoResponse, Context, Response};

/// A Tide endpoint.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Tide users.
///
/// In practice, endpoints are functions that take a `Context<State>` as an argument and
/// return a type `T` that implements [`IntoResponse`].
///
/// # Examples
///
/// Endpoints are implemented as asynchronous functions that make use of language features
/// currently only available in Rust Nightly. For this reason, we have to explicitly enable
/// those features with `#![feature(async_await)]`. To keep examples concise,
/// the attribute will be omitted in most of the documentation.
///
/// A simple endpoint that is invoked on a `GET` request and returns a `String`:
///
/// ```rust, no_run
/// # #![feature(async_await)]
/// async fn hello(_cx: tide::Context<()>) -> String {
///     String::from("hello")
/// }
///
/// fn main() {
///     let mut app = tide::App::new();
///     app.at("/hello").get(hello);
///     app.serve("127.0.0.1:8000").unwrap()
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
///     app.serve("127.0.0.1:8000").unwrap()
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

pub(crate) type DynEndpoint<State> =
    dyn (Fn(Context<State>) -> BoxFuture<'static, Response>) + 'static + Send + Sync;

impl<State, F: Send + Sync + 'static, Fut> Endpoint<State> for F
where
    F: Fn(Context<State>) -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: IntoResponse,
{
    type Fut = BoxFuture<'static, Response>;
    fn call(&self, cx: Context<State>) -> Self::Fut {
        let fut = (self)(cx);
        box_async! {
            fut.await.into_response()
        }
    }
}
