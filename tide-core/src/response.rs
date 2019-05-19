use http_service::Body;

pub type Response = http_service::Response;

/// Serialize `t` into a JSON-encoded response.
pub fn json<T: serde::Serialize>(t: T) -> Response {
    // TODO: remove the `unwrap`
    http::Response::builder()
        .status(http::status::StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&t).unwrap()))
        .unwrap()
}

/// A value that is synchronously convertable into a `Response`.
pub trait IntoResponse: Send + Sized {
    /// Convert the value into a `Response`.
    fn into_response(self) -> Response;

    /// Create a new `IntoResponse` value that will respond with the given status code.
    ///
    /// ```
    /// # use tide::response::IntoResponse;
    /// let resp = "Hello, 404!".with_status(http::status::StatusCode::NOT_FOUND).into_response();
    /// assert_eq!(resp.status(), http::status::StatusCode::NOT_FOUND);
    /// ```
    fn with_status(self, status: http::status::StatusCode) -> WithStatus<Self> {
        WithStatus {
            inner: self,
            status,
        }
    }
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
            .header("Content-Type", "application/octet-stream")
            .body(Body::from(self))
            .unwrap()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        http::Response::builder()
            .status(http::status::StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Body::from(self.into_bytes()))
            .unwrap()
    }
}

impl IntoResponse for &'_ str {
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

impl<T: Send + Into<Body>> IntoResponse for http::Response<T> {
    fn into_response(self) -> Response {
        self.map(Into::into)
    }
}

/// A response type that modifies the status code.
#[derive(Debug)]
pub struct WithStatus<R> {
    inner: R,
    status: http::status::StatusCode,
}

impl<R: IntoResponse> IntoResponse for WithStatus<R> {
    fn into_response(self) -> Response {
        let mut resp = self.inner.into_response();
        *resp.status_mut() = self.status;
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status() {
        let resp = "foo"
            .with_status(http::status::StatusCode::NOT_FOUND)
            .into_response();
        assert_eq!(resp.status(), http::status::StatusCode::NOT_FOUND);
    }

    #[test]
    fn byte_vec_content_type() {
        let resp = String::from("foo").into_bytes().into_response();
        assert_eq!(resp.headers()["Content-Type"], "application/octet-stream");
    }

    #[test]
    fn string_content_type() {
        let resp = String::from("foo").into_response();
        assert_eq!(resp.headers()["Content-Type"], "text/plain; charset=utf-8");
    }
}
