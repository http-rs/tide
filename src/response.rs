use crate::body;
use crate::body::Body;

/// An HTTP response.
///
/// A convenient alias for the `http::Response` type, using Tide's `Body`.
pub type Response = http::Response<Body>;

/// A value that is synchronously convertable into a `Response`.
pub trait IntoResponse: Send + 'static + Sized {
    fn into_response(self) -> Response;
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        http::Response::builder()
            .status(http::status::StatusCode::OK)
            .body(Body::empty())
            .unwrap()
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        http::Response::builder()
            .status(http::status::StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Body::from(self))
            .unwrap()
    }
}

impl IntoResponse for body::Bytes {
    fn into_response(self) -> Response {
        self.to_vec().into_response()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        self.into_bytes().into_response()
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        self.to_string().into_response()
    }
}

impl IntoResponse for http::status::StatusCode {
    fn into_response(self) -> Response {
        http::Response::builder()
            .status(self)
            .body(Body::empty())
            .unwrap()
    }
}

impl<T: IntoResponse, U: IntoResponse> IntoResponse for Result<T, U> {
    fn into_response(self) -> Response {
        match self {
            Ok(r) => r.into_response(),
            Err(r) => {
                let res = r.into_response();
                if res.status().is_success() {
                    panic!(
                        "Attempted to yield error response with success code {:?}",
                        res.status()
                    )
                }
                res
            }
        }
    }
}

impl<T: Send + 'static + Into<Body>> IntoResponse for http::Response<T> {
    fn into_response(self) -> Response {
        self.map(Into::into)
    }
}
