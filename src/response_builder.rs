use crate::http::headers::{HeaderName, ToHeaderValues};
use crate::http::{Body, Mime, StatusCode};
use crate::Response;
use std::convert::TryInto;

#[derive(Debug)]

/// Response Builder
///
/// Provides an ergonomic way to chain the creation of a response. This is generally accessed through [`Response::builder`](crate::Response::builder)
///
/// # Example
/// ```rust
/// # use tide::{StatusCode, Response, http::mime};
/// # async_std::task::block_on(async move {
/// let mut response = Response::builder(203)
///     .body("body")
///     .content_type(mime::HTML)
///     .header("custom-header", "value")
///     .build();
///
/// assert_eq!(response.take_body().into_string().await.unwrap(), "body");
/// assert_eq!(response.status(), StatusCode::NonAuthoritativeInformation);
/// assert_eq!(response["custom-header"], "value");
/// assert_eq!(response.content_type(), Some(mime::HTML));
/// # });

pub struct ResponseBuilder(Response);

impl ResponseBuilder {
    pub(crate) fn new<S>(status: S) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: std::fmt::Debug,
    {
        Self(Response::new(status))
    }

    /// Returns the inner Response
    pub fn build(self) -> Response {
        self.0
    }

    /// Sets a header on the response.
    /// ```
    /// # use tide::Response;
    /// let response = Response::builder(200).header("header-name", "header-value").build();
    /// assert_eq!(response["header-name"], "header-value");
    /// ```
    pub fn header(mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) -> Self {
        self.0.insert_header(key, value);
        self
    }

    /// Sets the Content-Type header on the response.
    /// ```
    /// # use tide::{http::mime, Response};
    /// let response = Response::builder(200).content_type(mime::HTML).build();
    /// assert_eq!(response["content-type"], "text/html;charset=utf-8");
    /// ```
    pub fn content_type(mut self, content_type: impl Into<Mime>) -> Self {
        self.0.set_content_type(content_type);
        self
    }

    /// Sets the body of the response.
    /// ```
    /// # async_std::task::block_on(async move {
    /// # use tide::{Response, convert::json};
    /// let mut response = Response::builder(200).body(json!({ "any": "Into<Body>"})).build();
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
        self.build()
    }
}

impl Into<crate::Result> for ResponseBuilder {
    fn into(self) -> crate::Result {
        Ok(self.build())
    }
}
