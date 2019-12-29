//! Tide error types.
use http_types::StatusCode;

use crate::response::{IntoResponse, Response};

/// A specialized Result type for Tide.
pub type Result<T = Response> = std::result::Result<T, Error>;

/// Error type.
pub use http_types::Error;

impl From<Response> for Error {
    fn from(resp: Response) -> Error {
        Error::from_str(resp.status(), "")
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Response::new(self.status())
            .set_header(
                http_types::headers::CONTENT_TYPE,
                "text/plain; charset=utf-8",
            )
            .body_string(self.to_string())
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
pub trait ResultExt<T>: Sized {
    /// Convert to an `Result`, treating the `Err` case as a client
    /// error (response code 400).
    fn client_err(self) -> Result<T> {
        self.with_err_status(StatusCode::BadRequest)
    }

    /// Convert to an `Result`, treating the `Err` case as a server
    /// error (response code 500).
    fn server_err(self) -> Result<T> {
        self.with_err_status(StatusCode::InternalServerError)
    }

    /// Convert to an `Result`, wrapping the `Err` case with a custom
    /// response status.
    fn with_err_status(self, status: StatusCode) -> Result<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T> for std::result::Result<T, E> {
    fn with_err_status(self, status: StatusCode) -> Result<T> {
        self.map_err(|err| Error::new(status, err))
    }
}
