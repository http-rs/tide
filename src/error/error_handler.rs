use crate::{Middleware, Next, Request, Result};
use std::fmt::{Debug, Display};
use std::future::Future;
use std::pin::Pin;

/// This trait maps from a specific error type to a tide::Result
/// future and is implemented for
/// `async Fn<E: Send + Sync + 'static>(E) -> tide::Result`
pub trait ErrorMapper<E>: Send + Sync + 'static {
    fn call<'a>(&'a self, error: E) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>>;
}

impl<E, F, Fut> ErrorMapper<E> for F
where
    E: Send + Sync + 'static,
    F: Fn(E) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result> + Send + 'static,
{
    fn call<'a>(&'a self, error: E) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
        Box::pin(async move { (self)(error).await })
    }
}

/// # Error Handler Middleware
///
/// This middleware provides a means of transforming specific errors
/// that have been returned from the endpoint or middleware before
/// it. An error handler function can either transform the downcast
/// error into a response or pass it along unchanged to the next
/// middleware.
///
/// ```rust
/// # use tide::http::{url::{self, Url}, Method, Request};
/// # use tide::{Response, StatusCode};
/// use tide::error::ErrorHandler;
///
/// let mut app = tide::new();
///
/// app.middleware(ErrorHandler::new(
///     |err: url::ParseError| async move {
///         let mut response = Response::new(StatusCode::ImATeapot);
///         response.set_body(err.to_string());
///         Ok(response)
///     },
/// ));
///
/// app.at("/").get(|_| async {
///     let _bad_parse = Url::parse("parse error")?; //intentionally errors
///     Ok("not reached")
/// });
///
/// # async_std::task::block_on(async move {
/// let response: Response = app
///     .respond(Request::new(
///         Method::Get,
///         Url::parse("http://example.com/")?,
///     ))
///     .await?;
///
/// assert_eq!(response.status(), StatusCode::ImATeapot);
/// # tide::Result::Ok(())
/// # }).unwrap();
/// ```

pub struct ErrorHandler<E>(Box<dyn ErrorMapper<E>>);

impl<E> ErrorHandler<E>
where
    E: Display + Debug + Send + Sync + 'static,
{
    pub fn new(f: impl ErrorMapper<E>) -> Self {
        Self(Box::new(f))
    }
}

impl<E> Debug for ErrorHandler<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "ErrorHandler for {}",
            std::any::type_name::<E>()
        ))
    }
}

impl<State: Send + Sync + 'static, E> Middleware<State> for ErrorHandler<E>
where
    E: Display + Debug + Send + Sync + 'static,
{
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
        Box::pin(async move {
            let response = next.run(req).await;
            if let Err(e) = response {
                match e.downcast::<E>() {
                    Ok(downcast) => self.0.call(downcast).await,
                    Err(e) => Err(e),
                }
            } else {
                response
            }
        })
    }

    fn name(&self) -> String {
        format!("ErrorHandler for {}", std::any::type_name::<E>())
    }
}
