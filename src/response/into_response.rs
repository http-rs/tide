use crate::{Request, Response};
use async_std::io::BufReader;
use http_types::StatusCode;

/// Conversion into a `Response`.
pub trait IntoResponse: Send + Sized {
    /// Convert the value into a `Response`.
    fn into_response(self) -> Response;

    /// Create a new `IntoResponse` value that will respond with the given status code.
    ///
    /// ```
    /// # use tide::IntoResponse;
    /// let resp = "Hello, 404!".with_status(http_types::StatusCode::NotFound).into_response();
    /// assert_eq!(resp.status(), http_types::StatusCode::NotFound);
    /// ```
    fn with_status(self, status: http_types::StatusCode) -> WithStatus<Self> {
        WithStatus {
            inner: self,
            status,
        }
    }
}

// impl IntoResponse for () {
//     fn into_response(self) -> Response {
//         http::Response::builder()
//             .status(http::status::StatusCode::NO_CONTENT)
//             .body(Body::empty())
//             .unwrap()
//     }
// }

// impl IntoResponse for Vec<u8> {
//     fn into_response(self) -> Response {
//         http::Response::builder()
//             .status(http::status::StatusCode::OK)
//             .header("Content-Type", "application/octet-stream")
//             .body(Body::from(self))
//             .unwrap()
//     }
// }

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::new(StatusCode::Ok)
            .set_header(
                http_types::headers::CONTENT_TYPE,
                "text/plain; charset=utf-8",
            )
            .body_string(self)
    }
}

impl<State: Send + Sync + 'static> IntoResponse for Request<State> {
    fn into_response(self) -> Response {
        Response::new(StatusCode::Ok).body(BufReader::new(self))
    }
}

impl IntoResponse for &'_ str {
    fn into_response(self) -> Response {
        self.to_string().into_response()
    }
}

// impl IntoResponse for http::status::StatusCode {
//     fn into_response(self) -> Response {
//         http::Response::builder()
//             .status(self)
//             .body(Body::empty())
//             .unwrap()
//     }
// }

// impl<T: IntoResponse, U: IntoResponse> IntoResponse for Result<T, U> {
//     fn into_response(self) -> Response {
//         match self {
//             Ok(r) => r.into_response(),
//             Err(r) => {
//                 let res = r.into_response();
//                 if res.status().is_success() {
//                     panic!(
//                         "Attempted to yield error response with success code {:?}",
//                         res.status()
//                     )
//                 }
//                 res
//             }
//         }
//     }
// }

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

/// A response type that modifies the status code.
#[derive(Debug)]
pub struct WithStatus<R> {
    inner: R,
    status: http_types::StatusCode,
}

impl<R: IntoResponse> IntoResponse for WithStatus<R> {
    fn into_response(self) -> Response {
        self.inner.into_response().set_status(self.status)
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(t) => t.into_response(),
            Err(e) => e.into_response(),
        }
    }
}
