use std::convert::TryInto;
use std::fmt::{Debug, Display};
use std::ops::Index;

use crate::http::cookies::Cookie;
use crate::http::headers::{self, HeaderName, HeaderValues, ToHeaderValues};
use crate::http::{self, Body, Error, Mime, StatusCode};

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
    pub fn new<S>(status: S) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
    {
        let res = http::Response::new(status);
        Self {
            res,
            cookie_events: vec![],
        }
    }

    /// Returns the statuscode.
    #[must_use]
    pub fn status(&self) -> crate::StatusCode {
        self.res.status()
    }

    /// Set the statuscode.
    #[must_use]
    pub fn set_status(&mut self, status: crate::StatusCode) {
        self.res.set_status(status);
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

    /// Get an HTTP header mutably.
    #[must_use]
    pub fn header_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut HeaderValues> {
        self.res.header_mut(name)
    }

    /// Remove a header.
    pub fn remove_header(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValues> {
        self.res.remove_header(name)
    }

    /// Insert an HTTP header.
    pub fn insert_header(&mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) {
        self.res.insert_header(key, value);
    }

    /// Append an HTTP header.
    pub fn append_header(&mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) {
        self.res.append_header(key, value);
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
    pub fn set_content_type(&mut self, mime: impl Into<Mime>) {
        self.res.set_content_type(mime.into());
    }

    /// Set the body reader.
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.res.set_body(body);
    }

    /// Take the response body as a `Body`.
    //
    // This method can be called after the body has already been taken or read,
    // but will return an empty `Body`.
    //
    // Useful for adjusting the whole body, such as in middleware.
    pub fn take_body(&mut self) -> Body {
        self.res.take_body()
    }

    /// Swaps the value of the body with another body, without deinitializing
    /// either one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use async_std::io::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    /// # async_std::task::block_on(async {
    /// #
    /// use tide::Response;
    ///
    /// let mut req = Response::new(200);
    /// req.set_body("Hello, Nori!");
    ///
    /// let mut body = "Hello, Chashu!".into();
    /// req.swap_body(&mut body);
    ///
    /// let mut string = String::new();
    /// body.read_to_string(&mut string).await?;
    /// assert_eq!(&string, "Hello, Nori!");
    /// #
    /// # Ok(()) }) }
    /// ```
    pub fn swap_body(&mut self, body: &mut Body) {
        self.res.swap_body(body)
    }

    /// Insert cookie in the cookie jar.
    pub fn insert_cookie(&mut self, cookie: Cookie<'static>) {
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

    /// Returns an optional reference to the `Error` if the response was created from one, or else `None`.
    pub fn error(&self) -> Option<&Error> {
        self.res.error()
    }

    pub fn downcast_error<E>(&self) -> Option<&E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        self.res.error()?.downcast_ref()
    }

    /// Takes the `Error` from the response if one exists, replacing it with `None`.
    pub fn take_error(&mut self) -> Option<Error> {
        self.res.take_error()
    }

    /// Get a response scoped extension value.
    #[must_use]
    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.res.ext().get()
    }

    /// Set a response scoped extension value.
    pub fn insert_ext<T: Send + Sync + 'static>(mut self, val: T) {
        self.res.ext_mut().insert(val);
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

impl From<http::Body> for Response {
    fn from(body: http::Body) -> Self {
        let mut res = Response::new(200);
        res.set_body(body);
        res
    }
}

impl From<serde_json::Value> for Response {
    fn from(json_value: serde_json::Value) -> Self {
        Body::from_json(&json_value)
            .map(|body| body.into())
            .unwrap_or_else(|_| Response::new(StatusCode::InternalServerError))
    }
}

impl From<Error> for Response {
    fn from(err: Error) -> Self {
        let res: http::Response = err.into();
        res.into()
    }
}

impl From<crate::Result> for Response {
    fn from(result: crate::Result) -> Self {
        match result {
            Ok(res) => res,
            Err(err) => err.into(),
        }
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
        Body::from_string(s).into()
    }
}

impl<'a> From<&'a str> for Response {
    fn from(s: &'a str) -> Self {
        Body::from_string(String::from(s)).into()
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
