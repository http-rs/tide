use core::pin::Pin;
use futures::future::Future;
use http::{HttpTryFrom, StatusCode};
use http_service::Body;

use crate::response::{IntoResponse, Response};

pub(crate) type BoxTryFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'static>>;

/// A specialized Result type for Tide operations.
pub type Result<T = Response> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct StringError(pub String);
impl std::error::Error for StringError {}

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

/// A generic error.
#[derive(Debug)]
pub struct Error {
    resp: Response,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.resp
    }
}

struct Cause(Box<dyn std::error::Error + Send + Sync>);

impl From<Response> for Error {
    fn from(resp: Response) -> Error {
        Error { resp }
    }
}

impl From<StatusCode> for Error {
    fn from(status: StatusCode) -> Error {
        Error { resp: Response::new(status) }
    }
}

/// Extends the `Result` type with convenient methods for constructing Tide errors.
pub trait ResultExt<T>: Sized {
    /// Convert to an `Result`, treating the `Err` case as a client
    /// error (response code 400).
    fn client_err(self) -> Result<T> {
        self.with_err_status(400)
    }

    /// Convert to an `Result`, treating the `Err` case as a server
    /// error (response code 500).
    fn server_err(self) -> Result<T> {
        self.with_err_status(500)
    }

    /// Convert to an `Result`, wrapping the `Err` case with a custom
    /// response status.
    fn with_err_status<S>(self, status: S) -> Result<T>
    where
        StatusCode: HttpTryFrom<S>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T> for std::result::Result<T, E> {
    fn with_err_status<S>(self, status: S) -> Result<T>
    where
        StatusCode: HttpTryFrom<S>,
    {
        self.map_err(|e| Error {
            resp: http::Response::builder()
                .status(status)
                .extension(Cause(Box::new(e)))
                .body(Body::empty())
                .unwrap()
                .into(),
        })
    }
}
