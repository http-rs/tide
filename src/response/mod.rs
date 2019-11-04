use http::StatusCode;
use http_service::Body;
pub use into_response::IntoResponse;

mod into_response;

/// An HTTP response
#[derive(Debug)]
pub struct Response {
    res: http_service::Response,
}

impl Response {
    /// Create a new instance.
    pub fn new(status: http::StatusCode) -> Self {
        let res = http::Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap();
        Self { res }
    }

    /// Returns the statuscode.
    pub fn status(&self) -> http::StatusCode {
        self.res.status()
    }

    /// Set the statuscode.
    pub fn set_status(mut self, status: http::StatusCode) -> Self {
        *self.res.status_mut() = status;
        self
    }

    /// Insert an HTTP header.
    pub fn insert_header(mut self, key: &'static str, value: impl AsRef<str>) -> Self {
        let value = value.as_ref().to_owned();
        self.res.headers_mut().insert(key, value.parse().unwrap());
        self
    }

    /// Encode a struct as a form and set as the response body.
    pub async fn body_form<T: serde::Serialize>(
        mut self,
        form: T,
    ) -> Result<Response, serde_qs::Error> {
        // TODO: think about how to handle errors
        *self.res.body_mut() = Body::from(serde_qs::to_string(&form)?.into_bytes());
        Ok(self
            .set_status(StatusCode::OK)
            .insert_header("Content-Type", "application/x-www-form-urlencoded"))
    }

    // fn body_multipart(&mut self) -> BoxTryFuture<Multipart<Cursor<Vec<u8>>>> {
    //     const BOUNDARY: &str = "boundary=";
    //     let boundary = self.headers().get("content-type").and_then(|ct| {
    //         let ct = ct.to_str().ok()?;
    //         let idx = ct.find(BOUNDARY)?;
    //         Some(ct[idx + BOUNDARY.len()..].to_string())
    //     });

    //     let body = self.take_body();

    //     Box::pin(async move {
    //         let body = body.into_vec().await.client_err()?;
    //         let boundary = boundary
    //             .ok_or_else(|| StringError(format!("no boundary found")))
    //             .client_err()?;
    //         Ok(Multipart::with_body(Cursor::new(body), boundary))
    //     })
    // }
}

#[doc(hidden)]
impl Into<http_service::Response> for Response {
    fn into(self) -> http_service::Response {
        self.res
    }
}

#[doc(hidden)]
impl From<http_service::Response> for Response {
    fn from(res: http_service::Response) -> Self {
        Self { res }
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
