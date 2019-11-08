use async_std::io::prelude::*;

use serde::Serialize;
use http::StatusCode;
use http_service::Body;
use mime::Mime;

pub use into_response::IntoResponse;

mod into_response;

/// An HTTP response
#[derive(Debug)]
pub struct Response {
    res: http_service::Response,
}

impl Response {
    /// Create a new instance.
    pub fn new(status: u16) -> Self {
        let status = http::StatusCode::from_u16(status).expect("invalid status code");
        let res = http::Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap();
        Self { res }
    }

    /// Create a new instance with an `200 OK` status code.
    pub fn ok() -> Self {
        Self::new(200)
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
    pub fn set_header(mut self, key: &'static str, value: impl AsRef<str>) -> Self {
        let value = value.as_ref().to_owned();
        self.res.headers_mut().insert(key, value.parse().unwrap());
        self
    }

    /// Set the request MIME.
    ///
    /// [Read more on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)
    pub fn set_mime(self, mime: Mime) -> Self {
        self.set_header("Content-Type", format!("{}", mime))
    }

    /// Pass a string as the request body.
    ///
    /// # Mime
    ///
    /// The encoding is set to `text/plain; charset=utf-8`.
    pub fn body_string(mut self, string: String) -> Self {
        *self.res.body_mut() = string.into_bytes().into();
        self.set_mime(mime::TEXT_PLAIN_UTF_8)
    }

    /// Pass a string as the request body.
    ///
    /// # Mime
    ///
    /// The encoding is set to `text/plain; charset=utf-8`.
    pub fn body<R>(mut self, reader: R) -> Self
    where
        R: Read + Unpin + Send + 'static,
    {
        *self.res.body_mut() = Box::new(reader).into();
        self.set_mime(mime::APPLICATION_OCTET_STREAM)
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
            .set_header("Content-Type", "application/x-www-form-urlencoded"))
    }

    /// Encode a struct as a form and set as the response body.
    pub fn body_json(mut self, json: &impl Serialize) -> serde_json::Result<Self> {
        *self.res.body_mut() = serde_json::to_vec(json)?.into();
        Ok(self.set_mime(mime::APPLICATION_JSON))
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
