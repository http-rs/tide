use crate::http::headers::{HeaderName, ToHeaderValues};
use crate::http::{Body, Mime, StatusCode};
use crate::Response;
use std::convert::TryInto;

#[derive(Debug)]

/// Response Builder
///
/// Provides an ergonomic way to chain the creation of a response. This is generally accessed through [`Response::build`](crate::Response::build)
///
/// # Example
/// ```rust
/// # use tide::{StatusCode, ResponseBuilder};
/// # async_std::task::block_on(async move {
/// let mut response = ResponseBuilder::new()
///     .body("body")
///     .status(203)
///     .header("custom-header", "value")
///     .unwrap();
///
/// assert_eq!(response.take_body().into_string().await.unwrap(), "body");
/// assert_eq!(response.status(), StatusCode::NonAuthoritativeInformation);
/// assert_eq!(response["custom-header"], "value");
/// # });

pub struct ResponseBuilder(Response);

impl ResponseBuilder {
    /// Creates a new ResponseBuilder. This will default to an 200 OK status code.
    /// ```rust
    /// # use tide::{StatusCode, ResponseBuilder};
    /// assert_eq!(ResponseBuilder::new().unwrap().status(), StatusCode::Ok)
    /// ```
    pub fn new() -> Self {
        Self(Response::new(StatusCode::Ok))
    }

    /// Returns the inner Response
    pub fn unwrap(self) -> Response {
        self.0
    }

    /// Sets the http status on the response.
    /// ```
    /// # use tide::{ResponseBuilder, StatusCode};
    /// let response = ResponseBuilder::new().status(418).unwrap();
    /// assert_eq!(response.status(), StatusCode::ImATeapot);
    /// ```
    /// # Panics:
    /// `status` will panic if the status argument cannot be successfully converted into a StatusCode.
    /// ```should_panic
    /// # use tide::ResponseBuilder;
    /// ResponseBuilder::new().status(210); // this is not an established status code and will panic
    /// ```
    pub fn status<S>(mut self, status: S) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: std::fmt::Debug,
    {
        self.0.set_status(status);
        self
    }

    /// Sets a header on the response.
    /// ```
    /// # use tide::ResponseBuilder;
    /// let response = ResponseBuilder::new().header("header-name", "header-value").unwrap();
    /// assert_eq!(response["header-name"], "header-value");
    /// ```
    pub fn header(mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) -> Self {
        self.0.insert_header(key, value);
        self
    }

    /// Sets a header on the response.
    /// ```
    /// # use tide::{http::mime, ResponseBuilder};
    /// let response = ResponseBuilder::new().content_type(mime::HTML).unwrap();
    /// assert_eq!(response["content-type"], "text/html;charset=utf-8");
    /// ```
    pub fn content_type(mut self, content_type: impl Into<Mime>) -> Self {
        self.0.set_content_type(content_type);
        self
    }

    /// Sets the body of the response.
    /// ```
    /// # async_std::task::block_on(async move {
    /// # use tide::{ResponseBuilder, convert::json};
    /// let mut response = ResponseBuilder::new().body(json!({ "any": "Into<Body>"})).unwrap();
    /// assert_eq!(response.take_body().into_string().await.unwrap(), "{\"any\":\"Into<Body>\"}");
    /// # });
    /// ```
    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.0.set_body(body);
        self
    }
}

impl Into<Response> for ResponseBuilder {
    fn into(self) -> Response {
        self.unwrap()
    }
}

impl Into<crate::Result> for ResponseBuilder {
    fn into(self) -> crate::Result {
        Ok(self.unwrap())
    }
}
