use crate::http::headers::{HeaderName, ToHeaderValues};
use crate::http::{Body, Mime, StatusCode};
use crate::Response;
use std::convert::TryInto;

#[derive(Debug)]
pub struct ResponseBuilder(Response);

impl ResponseBuilder {
    pub fn new() -> Self {
        Self(Response::new(StatusCode::Ok))
    }

    pub fn unwrap(self) -> Response {
        self.0
    }

    pub fn status<S>(mut self, status: S) -> Self
    where
        S: TryInto<StatusCode>,
        S::Error: std::fmt::Debug,
    {
        self.0.set_status(status);
        self
    }

    pub fn header(mut self, key: impl Into<HeaderName>, value: impl ToHeaderValues) -> Self {
        self.0.insert_header(key, value);
        self
    }

    pub fn content_type(mut self, content_type: impl Into<Mime>) -> Self {
        self.0.set_content_type(content_type);
        self
    }

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
