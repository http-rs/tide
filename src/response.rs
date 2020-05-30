use async_std::io::prelude::*;
use std::convert::TryFrom;
use std::ops::Index;

use serde::Serialize;

use crate::http::cookies::Cookie;
use crate::http::headers::{self, HeaderName, HeaderValues, ToHeaderValues};
use crate::http::{self, Body, StatusCode};
use crate::http::{mime, Mime};
use crate::redirect::Redirect;

#[derive(Debug)]
pub(crate) enum CookieEvent {
    Added(Cookie<'static>),
    Removed(Cookie<'static>),
}

/// An HTTP response
#[derive(Debug)]
pub struct Response {
    pub(crate) res: http::Response,
    // tracking here
    pub(crate) cookie_events: Vec<CookieEvent>,
}

impl Response {
    /// Create a new instance.
    #[must_use]
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

    /// Creates a response that represents a redirect to `location`.
    ///
    /// Uses status code 302 Found.
    ///
    /// # Example
    ///
    /// ```
    /// # use tide::{Response, Request, StatusCode};
    /// # fn special_sale_today() -> Option<String> { None }
    /// # #[allow(dead_code)]
    /// async fn route_handler(request: Request<()>) -> tide::Result {
    ///     if let Some(sale_url) = special_sale_today() {
    ///         Ok(Response::redirect(sale_url))
    ///     } else {
    ///         //...
    /// #       Ok(Response::new(StatusCode::Ok)) //...
    ///     }
    /// }
    /// ```
    pub fn redirect(location: impl AsRef<str>) -> Self {
        Redirect::new(location).into()
    }

    /// Returns the statuscode.
    #[must_use]
    pub fn status(&self) -> crate::StatusCode {
        self.res.status()
    }

    /// Set the statuscode.
    #[must_use]
    pub fn set_status(mut self, status: crate::StatusCode) -> Self {
        self.res.set_status(status);
        self
    }

    /// Get the length of the body.
    #[must_use]
    pub fn len(&self) -> Option<usize> {
        self.res.len()
    }

    /// Checks if the body is empty.
    #[must_use]
    pub fn is_empty(&self) -> Option<bool> {
        Some(self.res.len()? == 0)
    }

    /// Get an HTTP header.
    #[must_use]
    pub fn header(&self, name: impl Into<HeaderName>) -> Option<&HeaderValues> {
        self.res.header(name)
    }

    /// Remove a header.
    pub fn remove_header(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValues> {
        self.res.remove_header(name)
    }

    /// Insert an HTTP header.
    pub fn set_header(mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) -> Self {
        self.res.insert_header(key, value);
        self
    }

    /// Append an HTTP header.
    pub fn append_header(mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) -> Self {
        self.res.append_header(key, value);
        self
    }

    /// An iterator visiting all header pairs in arbitrary order.
    #[must_use]
    pub fn iter(&self) -> headers::Iter<'_> {
        self.res.iter()
    }

    /// An iterator visiting all header pairs in arbitrary order, with mutable references to the
    /// values.
    #[must_use]
    pub fn iter_mut(&mut self) -> headers::IterMut<'_> {
        self.res.iter_mut()
    }

    /// An iterator visiting all header names in arbitrary order.
    #[must_use]
    pub fn header_names(&self) -> headers::Names<'_> {
        self.res.header_names()
    }

    /// An iterator visiting all header values in arbitrary order.
    #[must_use]
    pub fn header_values(&self) -> headers::Values<'_> {
        self.res.header_values()
    }

    /// Get the response content type as a `Mime`.
    ///
    /// This gets the request `Content-Type` header.
    ///
    /// [Read more on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)
    #[must_use]
    pub fn content_type(&self) -> Option<Mime> {
        self.res.content_type()
    }

    /// Set the response content type from a `MIME`.
    ///
    /// This sets the response `Content-Type` header.
    ///
    /// [Read more on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)
    #[must_use]
    pub fn set_content_type(mut self, mime: impl Into<Mime>) -> Self {
        self.res.set_content_type(mime.into());
        self
    }

    /// Pass a string as the response body.
    ///
    /// # Mime
    ///
    /// The content type is set to `text/plain; charset=utf-8`.
    #[must_use]
    pub fn body_string(mut self, string: String) -> Self {
        self.res.set_body(string);
        self.set_content_type(mime::PLAIN)
    }

    /// Pass raw bytes as the response body.
    ///
    /// # Mime
    ///
    /// The content type is set to `application/octet-stream`.
    pub fn body<R>(mut self, reader: R) -> Self
    where
        R: BufRead + Unpin + Send + Sync + 'static,
    {
        self.res
            .set_body(http_types::Body::from_reader(reader, None));
        self.set_content_type(mime::BYTE_STREAM)
    }

    /// Set the body reader.
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.res.set_body(body);
    }

    /// Encode a struct as a form and set as the response body.
    ///
    /// # Mime
    ///
    /// The content type is set to `application/x-www-form-urlencoded`.
    pub async fn body_form<T: serde::Serialize>(
        mut self,
        form: T,
    ) -> Result<Self, serde_qs::Error> {
        // TODO: think about how to handle errors
        self.res.set_body(serde_qs::to_string(&form)?.into_bytes());
        Ok(self.set_status(StatusCode::Ok).set_content_type(mime::FORM))
    }

    /// Encode a struct as a form and set as the response body.
    ///
    /// # Mime
    ///
    /// The content type is set to `application/json`.
    pub fn body_json(mut self, json: &impl Serialize) -> serde_json::Result<Self> {
        self.res.set_body(serde_json::to_vec(json)?);
        Ok(self.set_content_type(mime::JSON))
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

    /// Take the response body as a `Body`.
    //
    // This method can be called after the body has already been taken or read,
    // but will return an empty `Body`.
    //
    // Useful for adjusting the whole body, such as in middleware.
    pub fn take_body(&mut self) -> Body {
        self.res.take_body()
    }

    /// Add cookie to the cookie jar.
    pub fn set_cookie(&mut self, cookie: Cookie<'static>) {
        self.cookie_events.push(CookieEvent::Added(cookie));
    }

    /// Removes the cookie. This instructs the `CookiesMiddleware` to send a cookie with empty value
    /// in the response.
    ///
    /// ## Warning
    /// Take care when calling this function with a cookie that was returned by
    /// [`Request::cookie`](Request::cookie).  As per [section 5.3 step 11 of RFC 6265], a new
    /// cookie is only treated as the same as an old one if it has a matching name, domain and
    /// path.
    ///
    /// The domain and path are not sent to the server on subsequent HTTP requests, so if a cookie
    /// was originally set with a domain and/or path, calling this function on a cookie with the
    /// same name but with either a different, or no, domain and/or path will lead to us sending an
    /// empty cookie that the user agent will treat as unrelated to the original one, and will thus
    /// not remove the old one.
    ///
    /// To avoid this you can manually set the [domain](Cookie::set_domain) and
    /// [path](Cookie::set_path) as necessary after retrieving the cookie using
    /// [`Request::cookie`](Request::cookie).
    ///
    /// [section 5.3 step 11 of RFC 6265]: https://tools.ietf.org/html/rfc6265#section-5.3
    pub fn remove_cookie(&mut self, cookie: Cookie<'static>) {
        self.cookie_events.push(CookieEvent::Removed(cookie));
    }

    /// Get a response scoped extension value.
    #[must_use]
    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.res.ext().get()
    }

    /// Set a response scoped extension value.
    pub fn set_ext<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.res.ext_mut().insert(val);
        self
    }

    /// Create a `tide::Response` from a type that can be converted into an
    /// `http_types::Response`.
    pub fn from_res<T>(value: T) -> Self
    where
        T: Into<http_types::Response>,
    {
        let res: http_types::Response = value.into();
        Self {
            res,
            cookie_events: vec![],
        }
    }
}

impl AsRef<http::Response> for Response {
    fn as_ref(&self) -> &http::Response {
        &self.res
    }
}

impl AsMut<http::Response> for Response {
    fn as_mut(&mut self) -> &mut http::Response {
        &mut self.res
    }
}

impl AsRef<http::Headers> for Response {
    fn as_ref(&self) -> &http::Headers {
        self.res.as_ref()
    }
}

impl AsMut<http::Headers> for Response {
    fn as_mut(&mut self) -> &mut http::Headers {
        self.res.as_mut()
    }
}

impl Into<http::Response> for Response {
    fn into(self) -> http_types::Response {
        self.res
    }
}

impl From<http::Response> for Response {
    fn from(res: http::Response) -> Self {
        Self {
            res,
            cookie_events: vec![],
        }
    }
}

impl From<String> for Response {
    fn from(s: String) -> Self {
        let mut res = http_types::Response::new(StatusCode::Ok);
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
        res.set_body(String::from(s));
        Self {
            res,
            cookie_events: vec![],
        }
    }
}

impl IntoIterator for Response {
    type Item = (HeaderName, HeaderValues);
    type IntoIter = http_types::headers::IntoIter;

    /// Returns a iterator of references over the remaining items.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.into_iter()
    }
}

impl<'a> IntoIterator for &'a Response {
    type Item = (&'a HeaderName, &'a HeaderValues);
    type IntoIter = http_types::headers::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.iter()
    }
}

impl<'a> IntoIterator for &'a mut Response {
    type Item = (&'a HeaderName, &'a mut HeaderValues);
    type IntoIter = http_types::headers::IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.res.iter_mut()
    }
}

impl Index<HeaderName> for Response {
    type Output = HeaderValues;

    /// Returns a reference to the value corresponding to the supplied name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not present in `Response`.
    #[inline]
    fn index(&self, name: HeaderName) -> &HeaderValues {
        &self.res[name]
    }
}

impl Index<&str> for Response {
    type Output = HeaderValues;

    /// Returns a reference to the value corresponding to the supplied name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not present in `Response`.
    #[inline]
    fn index(&self, name: &str) -> &HeaderValues {
        &self.res[name]
    }
}
