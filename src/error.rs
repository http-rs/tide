//! Tide error types.
use http::{HttpTryFrom, StatusCode};

use crate::response::{IntoResponse, Response};

/// A specialized Result type for Tide.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error which holds a response.
#[derive(Debug)]
pub struct Error {
    resp: Response,
}

impl Error {
    /// Create an `Error` with the given response.
    pub fn from_response(resp: Response) -> Error {
        Error { resp }
    }

    /// Create an `Error` with an empty response and the given status code.
    pub fn from_status(status: StatusCode) -> Error {
        Error {
            resp: Response::new(status.as_u16()),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.resp
    }
}

impl<E> From<E> for Error
where
    E: std::fmt::Display,
{
    fn from(err: E) -> Error {
        Error {
            resp: Response::new(500).body_string(err.to_string()),
        }
    }
}

/// A simple error type that wraps a String
#[derive(Debug)]
pub struct StringError(pub String);
impl std::error::Error for StringError {}

impl std::fmt::Display for StringError {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        self.0.fmt(formatter)
    }
}

/// Extension methods for `Result`.
pub trait ResultExt<T, E>: Sized {
    /// Convert a `Result<T, E>` into a `Result<T, Error>` using the function `op` to generate a
    /// response on an error.
    fn with_err_res<F, R>(self, op: F) -> std::result::Result<T, Error>
    where
        F: FnOnce(E) -> R,
        R: IntoResponse;

    /// Convert a `Result<T, E>` into a `Result<T, Error>` which generates an empty response with
    /// the given status code on an error.
    fn with_empty<S>(self, status: S) -> std::result::Result<T, Error>
    where
        StatusCode: HttpTryFrom<S>,
    {
        let status = StatusCode::try_from(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        self.err_res(Response::new(status.as_u16()))
    }

    /// Convert a `Result<T, E>` into a `Result<T, Error>` which generates a response using `E`'s
    /// `Display` implementation and the given status code on an error.
    fn with_status<S>(self, status: S) -> std::result::Result<T, Error>
    where
        StatusCode: HttpTryFrom<S>,
        E: std::fmt::Display,
    {
        let status = StatusCode::try_from(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        self.with_err_res(|err| Response::new(status.as_u16()).body_string(err.to_string()))
    }

    /// Convert a `Result<T, E>` into a `Result<T, Error>` which uses `response` for the response
    /// on an error.
    fn err_res<R>(self, response: R) -> std::result::Result<T, Error>
    where
        R: IntoResponse,
    {
        self.with_err_res(|_| response)
    }

    /// Convert a `Result<T, E>` into a `Result<T, Error>` using `E`'s `IntoResponse`
    /// implementation to generate a response on an error.
    fn err_into_res(self) -> std::result::Result<T, Error>
    where
        E: IntoResponse,
    {
        self.with_err_res(|e| e)
    }
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E> {
    fn with_err_res<F, R>(self, op: F) -> std::result::Result<T, Error>
    where
        F: FnOnce(E) -> R,
        R: IntoResponse,
    {
        self.map_err(|e| Error {
            resp: op(e).into_response(),
        })
    }
}
