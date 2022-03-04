use serde::Serialize;

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
    ///
    /// # Examples
    ///
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
    ///
    /// # Examples
    ///
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

    /// Pass JSON as the response body.
    ///
    /// # Mime
    ///
    /// The `Content-Type` is set to `application/json`.
    ///
    /// # Errors
    ///
    /// This method will return an error if the provided data could not be serialized to JSON.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use serde::{Deserialize, Serialize};
    /// # use tide::Response;
    /// # #[async_std::main]
    /// # async fn main() -> tide::Result<()> {
    /// #[derive(Deserialize, Serialize)]
    /// struct Ip {
    ///     ip: String
    /// }
    ///
    /// let data = &Ip { ip: "129.0.0.1".into() };
    /// let res = Response::builder(200).body_json(data)?.build();
    /// assert_eq!(res.status(), 200);
    /// # Ok(()) }
    /// ```
    pub fn body_json(self, json: &impl Serialize) -> crate::Result<Self> {
        Ok(self.body(Body::from_json(json)?))
    }

    /// Pass a string as the response body.
    ///
    /// # Mime
    ///
    /// The `Content-Type` is set to `text/plain; charset=utf-8`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tide::Response;
    /// # #[async_std::main]
    /// # async fn main() -> tide::Result<()> {
    /// let data = "hello world".to_string();
    /// let res = Response::builder(200).body_string(data).build();
    /// assert_eq!(res.status(), 200);
    /// # Ok(()) }
    /// ```
    pub fn body_string(self, string: String) -> Self {
        self.body(Body::from_string(string))
    }

    /// Pass bytes as the response body.
    ///
    /// # Mime
    ///
    /// The `Content-Type` is set to `application/octet-stream`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tide::Response;
    /// # #[async_std::main]
    /// # async fn main() -> tide::Result<()> {
    /// let data = b"hello world".to_owned();
    /// let res = Response::builder(200).body_bytes(data).build();
    /// assert_eq!(res.status(), 200);
    /// # Ok(()) }
    /// ```
    pub fn body_bytes(self, bytes: impl AsRef<[u8]>) -> Self {
        self.body(Body::from(bytes.as_ref()))
    }

    /// Pass a file as the response body.
    ///
    /// # Mime
    ///
    /// The `Content-Type` is set based on the file extension using [`mime_guess`] if the operation was
    /// successful. If `path` has no extension, or its extension has no known MIME type mapping,
    /// then `None` is returned.
    ///
    /// [`mime_guess`]: https://docs.rs/mime_guess
    ///
    /// # Errors
    ///
    /// This method will return an error if the file couldn't be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tide::Response;
    /// # #[async_std::main]
    /// # async fn main() -> tide::Result<()> {
    /// let res = Response::builder(200).body_file("./archive.tgz").await?.build();
    /// assert_eq!(res.status(), 200);
    /// # Ok(()) }
    /// ```
    #[cfg(not(feature = "wasm"))]
    pub async fn body_file(self, path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        Ok(self.body(Body::from_file(path).await?))
    }
}

impl From<ResponseBuilder> for Response {
    fn from(response_builder: ResponseBuilder) -> Response {
        response_builder.build()
    }
}
