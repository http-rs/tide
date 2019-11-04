pub use into_response::IntoResponse;
use http_service::Body;

mod into_response;

/// An HTTP Response
#[derive(Debug)]
pub struct Response {
    res: http_service::Response,
}

impl Response {
    /// Create a new instance.
    pub fn new(status: http::StatusCode) -> Self {
        http::Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }

    /// Returns the statuscode.
    pub fn status(&self) -> http::StatusCode {
        self.res.status()
    }

    /// Returns a mutable reference to the associated header field map.
    pub fn headers_mut(&mut self) -> &mut http::header::HeaderMap<http::header::HeaderValue> {
        self.res.headers_mut()
    }
}

// /// Serialize `t` into a JSON-encoded response.
// pub fn json<T: serde::Serialize>(t: T) -> Response {
//     let mut res = http::Response::builder();
//     match serde_json::to_vec(&t) {
//         Ok(v) => res
//             .header("Content-Type", "application/json")
//             .body(Body::from(v))
//             .unwrap(),
//         Err(e) => {
//             log::error!("{}", e);
//             res.status(http::status::StatusCode::INTERNAL_SERVER_ERROR)
//                 .body(Body::empty())
//                 .unwrap()
//         }
//     }
// }
