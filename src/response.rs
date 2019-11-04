use http_service::Body;

pub type Response = http_service::Response;

/// Serialize `t` into a JSON-encoded response.
pub fn json<T: serde::Serialize>(t: T) -> Response {
    let mut res = http::Response::builder();
    match serde_json::to_vec(&t) {
        Ok(v) => res
            .header("Content-Type", "application/json")
            .body(Body::from(v))
            .unwrap(),
        Err(e) => {
            log::error!("{}", e);
            res.status(http::status::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        }
    }
}

/// Conversion into a `Response`.
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
            .status(http::status::StatusCode::NO_CONTENT)
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
    use futures::executor::block_on;

    #[test]
    fn test_status() {
        let resp = "foo"
            .with_status(http::status::StatusCode::NOT_FOUND)
            .into_response();
        assert_eq!(resp.status(), http::status::StatusCode::NOT_FOUND);
        assert_eq!(block_on(resp.into_body().into_vec()).unwrap(), b"foo");
    }

    #[test]
    fn byte_vec_content_type() {
        let resp = String::from("foo").into_bytes().into_response();
        assert_eq!(resp.headers()["Content-Type"], "application/octet-stream");
        assert_eq!(block_on(resp.into_body().into_vec()).unwrap(), b"foo");
    }

    #[test]
    fn string_content_type() {
        let resp = String::from("foo").into_response();
        assert_eq!(resp.headers()["Content-Type"], "text/plain; charset=utf-8");
        assert_eq!(block_on(resp.into_body().into_vec()).unwrap(), b"foo");
    }

    #[test]
    fn json_content_type() {
        use std::collections::BTreeMap;

        let mut map = BTreeMap::new();
        map.insert(Some("a"), 2);
        map.insert(Some("b"), 4);
        map.insert(None, 6);

        let resp = json(map);
        assert_eq!(
            resp.status(),
            http::status::StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(block_on(resp.into_body().into_vec()).unwrap(), b"");

        let mut map = BTreeMap::new();
        map.insert("a", 2);
        map.insert("b", 4);
        map.insert("c", 6);

        let resp = json(map);
        assert_eq!(resp.status(), http::status::StatusCode::OK);
        assert_eq!(
            block_on(resp.into_body().into_vec()).unwrap(),
            br##"{"a":2,"b":4,"c":6}"##
        );
    }
}
