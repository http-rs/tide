#[macro_export]
macro_rules! err_fmt {
    {$($t:tt)*} => {
        $crate::error::StringError(format!($($t)*))
    }
}

pub use ext::{ResponseExt, ResultDynErrExt, ResultExt};
pub use types::{Cause, Error, StringError};

mod types {
    use http::StatusCode;
    use http_service::{Body, Response};

    use crate::response::IntoResponse;

    /// A generic endpoint error, which can be converted into a response.
    #[derive(Debug)]
    pub struct Error {
        resp: Response,
    }

    impl Error {
        pub fn new(response: Response) -> Self {
            Self { resp: response }
        }

        pub fn response_ref(&self) -> &Response {
            &self.resp
        }

        pub fn into_response(self) -> Response {
            self.resp
        }
    }

    impl IntoResponse for Error {
        fn into_response(self) -> Response {
            self.resp
        }
    }

    #[derive(Debug)]
    pub struct Cause(Box<dyn std::error::Error + Send + Sync>);

    impl Cause {
        pub fn new(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
            Self(error)
        }

        pub fn inner_ref(&self) -> &Box<dyn std::error::Error + Send + Sync> {
            &self.0
        }

        pub fn into_inner(self) -> Box<dyn std::error::Error + Send + Sync> {
            self.0
        }
    }

    impl From<Response> for Error {
        fn from(resp: Response) -> Error {
            Error { resp }
        }
    }

    impl From<StatusCode> for Error {
        fn from(status: StatusCode) -> Error {
            let resp = http::Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap();
            Error { resp }
        }
    }

    #[derive(Debug)]
    pub struct StringError(pub String);
    impl std::error::Error for StringError {}

    impl std::fmt::Display for StringError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
            self.0.fmt(f)
        }
    }
}

mod ext {
    use http::{HttpTryFrom, Response, StatusCode};
    use http_service::Body;

    use super::types::{Cause, Error};
    use crate::endpoint::EndpointResult;

    /// Extends the `Response` type with a method to extract error causes when applicable.
    pub trait ResponseExt {
        /// Extract the cause of the unsuccessful response, if any
        fn err_cause(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)>;
    }

    impl<T> ResponseExt for Response<T> {
        fn err_cause(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
            self.extensions().get().map(|c: &Cause| &**c.inner_ref())
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

        /// Convert to an `EndpointResult`, wrapping the `Err` case with a custom response status.
        fn with_err_status<S>(self, status: S) -> EndpointResult<T>
        where
            StatusCode: HttpTryFrom<S>;
    }

    impl<T, E: std::error::Error + Send + Sync + 'static> ResultExt<T> for std::result::Result<T, E> {
        fn with_err_status<S>(self, status: S) -> EndpointResult<T>
        where
            StatusCode: HttpTryFrom<S>,
        {
            let r = self.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
            r.with_err_status(status)
        }
    }

    /// Extends the `Result` type using `std::error::Error` trait object as the error type with
    /// convenient methods for constructing Tide errors.
    pub trait ResultDynErrExt<T>: Sized {
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

        /// Convert to an `EndpointResult`, wrapping the `Err` case with a custom response status.
        fn with_err_status<S>(self, status: S) -> EndpointResult<T>
        where
            StatusCode: HttpTryFrom<S>;
    }

    impl<T> ResultDynErrExt<T> for std::result::Result<T, Box<dyn std::error::Error + Send + Sync>> {
        fn with_err_status<S>(self, status: S) -> EndpointResult<T>
        where
            StatusCode: HttpTryFrom<S>,
        {
            self.map_err(|e| {
                Error::new(
                    Response::builder()
                        .status(status)
                        .extension(Cause::new(e))
                        .body(Body::empty())
                        .unwrap(),
                )
            })
        }
    }
}
