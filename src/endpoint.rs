use futures::future::{Future, FutureObj};
use std::pin::Pin;

use crate::{response::IntoResponse, Context, Response};

/// A Tide endpoint.
///
/// This trait is automatically implemented for `Fn` types, and so is rarely implemented
/// directly by Tide users.
///
/// In practice, endpoints are function that take a `Context<AppData>` as an argument and
/// return a type `T` that implements [`IntoResponse`].
///
/// # Examples
///
/// Endpoints are implemented as asynchronous functions that make use of language features
/// currently only available in Rust Nightly. For this reason, we have to explicitly enable
/// those features with `#![feature(async_await, futures_api)]`. To keep examples concise,
/// the attribute will be omitted in most of the documentation.
///
/// A simple endpoint that is invoked on a `GET` request and returns a `String`:
///
/// ```rust, no_run
/// # #![feature(async_await, futures_api)]
/// async fn hello(_cx: tide::Context<()>) -> String {
///     String::from("hello")
/// }
///
/// fn main() {
///     let mut app = tide::App::new(());
///     app.at("/hello").get(hello);
///     app.serve("127.0.0.1:8000").unwrap()
/// }
/// ```
pub trait Endpoint<AppData>: Send + Sync + 'static {
    /// The async result of `call`.
    type Fut: Future<Output = Response> + Send + 'static;

    /// Invoke the endpoint within the given context
    fn call(&self, cx: Context<AppData>) -> Self::Fut;
}

pub(crate) type DynEndpoint<AppData> =
    dyn (Fn(Context<AppData>) -> FutureObj<'static, Response>) + 'static + Send + Sync;

impl<AppData, F: Send + Sync + 'static, Fut> Endpoint<AppData> for F
where
    F: Fn(Context<AppData>) -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: IntoResponse,
{
    type Fut = ResponseWrapper<Fut>;
    fn call(&self, cx: Context<AppData>) -> Self::Fut {
        ResponseWrapper { fut: (self)(cx) }
    }
}

/// The future retured by the endpoint implementation for `Fn` types.
pub struct ResponseWrapper<F> {
    fut: F,
}

impl<F> Future for ResponseWrapper<F>
where
    F: Future,
    F::Output: IntoResponse,
{
    type Output = Response;

    fn poll(self: Pin<&mut Self>, waker: &std::task::Waker) -> std::task::Poll<Response> {
        let inner = unsafe { self.map_unchecked_mut(|wrapper| &mut wrapper.fut) };
        inner.poll(waker).map(IntoResponse::into_response)
    }
}
