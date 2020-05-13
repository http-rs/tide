use async_std::io::prelude::*;
use std::convert::TryFrom;

use cookie::Cookie;
use http_service::Body;
use http_types::{
    headers::{HeaderName, HeaderValue},
    StatusCode,
};
use mime::Mime;
use serde::Serialize;

#[derive(Debug)]
pub(crate) enum CookieEvent {
    Added(Cookie<'static>),
    Removed(Cookie<'static>),
}

/// An HTTP response
#[derive(Debug)]
pub struct Response {
    pub(crate) res: http_service::Response,
    // tracking here
    pub(crate) cookie_events: Vec<CookieEvent>,
}

impl Response {
    /// Create a new instance.
    pub fn new(status: StatusCode) -> Self {
        let res = http_types::Response::new(status);
        Self {
            res,
            cookie_events: vec![],
        }
    }

    /// Create a new instance from a reader.
    pub fn with_reader<R>(status: u16, reader: R) -> Self
    where
        R: BufRead + Unpin + Send + Sync + 'static,
    {
        let status = crate::StatusCode::try_from(status).expect("invalid status code");
        let mut res = http_types::Response::new(status);
        res.set_body(Body::from_reader(reader, None));

        Self {
            res,
            cookie_events: vec![],
        }
    }

    /// Creates a response that represents a permanent redirect to `location`.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use tide::{Response, Request, StatusCode};
    /// # fn canonicalize(uri: &url::Url) -> Option<&url::Url> { None }
    /// # #[allow(dead_code)]
    /// async fn route_handler(request: Request<()>) -> tide::Result {
    ///     if let Some(canonical_redirect) = canonicalize(request.uri()) {
    ///         Ok(Response::redirect_permanent(canonical_redirect))
    ///     } else {
    ///          //...
    /// #        Ok(Response::new(StatusCode::Ok)) // ...
    ///     }
    /// }
    /// ```
    pub fn redirect_permanent(location: impl AsRef<str>) -> Self {
        Response::new(StatusCode::PermanentRedirect)
            .set_header("location".parse().unwrap(), location)
    }

    /// Creates a response that represents a temporary redirect to `location`.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use tide::{Response, Request, StatusCode};
    /// # fn special_sale_today() -> Option<String> { None }
    /// # #[allow(dead_code)]
    /// async fn route_handler(request: Request<()>) -> tide::Result {
    ///     if let Some(sale_url) = special_sale_today() {
    ///         Ok(Response::redirect_temporary(sale_url))
    ///     } else {
    ///         //...
    /// #       Ok(Response::new(StatusCode::Ok)) //...
    ///     }
    /// }
    /// ```
    pub fn redirect_temporary(location: impl AsRef<str>) -> Self {
        Response::new(StatusCode::TemporaryRedirect)
            .set_header("location".parse().unwrap(), location)
    }

    /// Returns the statuscode.
    pub fn status(&self) -> crate::StatusCode {
        self.res.status()
    }

    /// Set the statuscode.
    pub fn set_status(mut self, status: crate::StatusCode) -> Self {
        self.res.set_status(status);
        self
    }

    /// Get the length of the body.
    pub fn len(&self) -> Option<usize> {
        self.res.len()
    }

    /// Insert an HTTP header.
    pub fn set_header(
        mut self,
        key: http_types::headers::HeaderName,
        value: impl AsRef<str>,
    ) -> Self {
        let value = value.as_ref().to_owned();
        self.res
            .insert_header(key, &[value.parse().unwrap()][..])
            .expect("invalid header");
        self
    }

    /// Append an HTTP header.
    pub fn append_header(
        mut self,
        key: http_types::headers::HeaderName,
        value: impl AsRef<str>,
    ) -> Self {
        let value = value.as_ref().to_owned();
        self.res
            .append_header(key, &[value.parse().unwrap()][..])
            .expect("invalid header");
        self
    }

    /// Set the request MIME.
    ///
    /// [Read more on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)
    pub fn set_mime(self, mime: Mime) -> Self {
        self.set_header(http_types::headers::CONTENT_TYPE, format!("{}", mime))
    }

    /// Pass a string as the request body.
    ///
    /// # Mime
    ///
    /// The encoding is set to `text/plain; charset=utf-8`.
    pub fn body_string(mut self, string: String) -> Self {
        self.res.set_body(string);
        self.set_mime(mime::TEXT_PLAIN_UTF_8)
    }

    /// Pass raw bytes as the request body.
    ///
    /// # Mime
    ///
    /// The encoding is set to `application/octet-stream`.
    pub fn body<R>(mut self, reader: R) -> Self
    where
        R: BufRead + Unpin + Send + Sync + 'static,
    {
        self.res
            .set_body(http_types::Body::from_reader(reader, None));
        self.set_mime(mime::APPLICATION_OCTET_STREAM)
    }

    /// Set the body reader.
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.res.set_body(body);
    }

    /// Encode a struct as a form and set as the response body.
    ///
    /// # Mime
    ///
    /// The encoding is set to `application/x-www-form-urlencoded`.
    pub async fn body_form<T: serde::Serialize>(
        mut self,
        form: T,
    ) -> Result<Response, serde_qs::Error> {
        // TODO: think about how to handle errors
        self.res.set_body(serde_qs::to_string(&form)?.into_bytes());
        Ok(self
            .set_status(StatusCode::Ok)
            .set_mime(mime::APPLICATION_WWW_FORM_URLENCODED))
    }

    /// Encode a struct as a form and set as the response body.
    ///
    /// # Mime
    ///
    /// The encoding is set to `application/json`.
    pub fn body_json(mut self, json: &impl Serialize) -> serde_json::Result<Self> {
        self.res.set_body(serde_json::to_vec(json)?);
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

    /// Take the request body, replacing it with an empty body.
    pub fn take_body(&mut self) -> Body {
        self.res.take_body()
    }

    /// Add cookie to the cookie jar.
    pub fn set_cookie(&mut self, cookie: Cookie<'static>) {
        self.cookie_events.push(CookieEvent::Added(cookie));
    }

    /// Removes the cookie. This instructs the `CookiesMiddleware` to send a cookie with empty value
    /// in the response.
    pub fn remove_cookie(&mut self, cookie: Cookie<'static>) {
        self.cookie_events.push(CookieEvent::Removed(cookie));
    }

    /// Get a local value.
    pub fn local<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.res.local().get()
    }

    /// Set a local value.
    pub fn set_local<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.res.local_mut().insert(val);
        self
    }
}

impl Into<http_service::Response> for Response {
    fn into(self) -> http_service::Response {
        self.res
    }
}

impl From<http_service::Response> for Response {
    fn from(res: http_service::Response) -> Self {
        Self {
            res,
            cookie_events: vec![],
        }
    }
}

impl From<String> for Response {
    fn from(s: String) -> Self {
        let mut res = http_types::Response::new(StatusCode::Ok);
        res.set_content_type(http_types::mime::PLAIN);
        res.set_body(s);
        Self {
            res,
            cookie_events: vec![],
        }
    }
}

impl<'a> From<&'a str> for Response {
    fn from(s: &'a str) -> Self {
        let mut res = http_types::Response::new(StatusCode::Ok);
        res.set_content_type(http_types::mime::PLAIN);
        res.set_body(String::from(s));
        Self {
            res,
            cookie_events: vec![],
        }
    }
}

impl IntoIterator for Response {
    type Item = (HeaderName, Vec<HeaderValue>);
    type IntoIter = http_types::headers::IntoIter;

    /// Returns a iterator of references over the remaining items.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.into_iter()
    }
}

impl<'a> IntoIterator for &'a Response {
    type Item = (&'a HeaderName, &'a Vec<HeaderValue>);
    type IntoIter = http_types::headers::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.iter()
    }
}

impl<'a> IntoIterator for &'a mut Response {
    type Item = (&'a HeaderName, &'a mut Vec<HeaderValue>);
    type IntoIter = http_types::headers::IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.iter_mut()
    }
}
