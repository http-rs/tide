use core::pin::Pin;
use futures::future::Future;
use http::{HttpTryFrom, Response, StatusCode};
use http_service::Body;

use crate::response::IntoResponse;

pub(crate) type BoxTryFuture<T> = Pin<Box<dyn Future<Output = EndpointResult<T>> + Send + 'static>>;

/// A convenient `Result` instantiation appropriate for most endpoints.
pub type EndpointResult<T = Response<Body>> = Result<T, Error>;

#[derive(Debug)]
pub struct StringError(pub String);
impl std::error::Error for StringError {}

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

macro_rules! err_fmt {
    {$($t:tt)*} => {
        crate::error::StringError(format!($($t)*))
    }
}

/// A generic endpoint error, which can be converted into a response.
#[derive(Debug)]
pub struct Error {
    resp: Response<Body>,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        self.resp
    }
}

struct Cause(Box<dyn std::error::Error + Send + Sync>);

impl From<Response<Body>> for Error {
    fn from(resp: Response<Body>) -> Error {
        Error { resp }
    }
}

impl From<StatusCode> for Error {
    fn from(status: StatusCode) -> Error {
        let resp = Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap();
        Error { resp }
    }
}

/// Extends the `Result` type with convenient methods for constructing Tide errors.
pub trait ResultExt<T>: Sized {
    /// Convert to an `EndpointResult`, treating the `Err` case as a client
    /// error (response code 400).
    fn client_err(self) -> EndpointResult<T> {
        self.with_err_status(400)
    }

    /// Convert to an `EndpointResult`, treating the `Err` case as a server
    /// error (response code 500).
    fn server_err(self) -> EndpointResult<T> {
        self.with_err_status(500)
    }

    /// Convert to an `EndpointResult`, wrapping the `Err` case with a custom
    /// response status.
    fn with_err_status<S>(self, status: S) -> EndpointResult<T>
    where
        StatusCode: HttpTryFrom<S>;
}

/// Extends the `Response` type with a method to extract error causes when applicable.
pub trait ResponseExt {
    /// Extract the cause of the unsuccessful response, if any
    fn err_cause(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)>;
}

impl<T> ResponseExt for Response<T> {
    fn err_cause(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.extensions().get().map(|Cause(c)| &**c)
    }
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T> for std::result::Result<T, E> {
    fn with_err_status<S>(self, status: S) -> EndpointResult<T>
    where
        StatusCode: HttpTryFrom<S>,
    {
        self.map_err(|e| Error {
            resp: Response::builder()
                .status(status)
                .extension(Cause(Box::new(e)))
                .body(Body::empty())
                .unwrap(),
        })
    }
}
