use http::{HeaderMap, Method, Uri, Version};
use http_service::Body;
use route_recognizer::Params;
use serde::Deserialize;

use async_std::io::{self, prelude::*};
use async_std::task::{Context, Poll};

use std::pin::Pin;
use std::{str::FromStr, sync::Arc};

pin_project_lite::pin_project! {
    /// An HTTP request.
    ///
    /// The `Request` gives endpoints access to basic information about the incoming
    /// request, route parameters, and various ways of accessing the request's body.
    ///
    /// Requests also provide *extensions*, a type map primarily used for low-level
    /// communication between middleware and endpoints.
    #[derive(Debug)]
    pub struct Request<State> {
        pub(crate) state: Arc<State>,
        #[pin]
        pub(crate) request: http_service::Request,
        pub(crate) route_params: Vec<Params>,
    }
}

impl<State> Request<State> {
    pub(crate) fn new(
        state: Arc<State>,
        request: http::Request<Body>,
        route_params: Vec<Params>,
    ) -> Request<State> {
        Request {
            state,
            request,
            route_params,
        }
    }

    /// Access the request's HTTP method.
    pub fn method(&self) -> &Method {
        self.request.method()
    }

    /// Access the request's full URI method.
    pub fn uri(&self) -> &Uri {
        self.request.uri()
    }

    /// Access the request's HTTP version.
    pub fn version(&self) -> Version {
        self.request.version()
    }

    /// Access the request's headers.
    pub fn headers(&self) -> &HeaderMap {
        self.request.headers()
    }

    /// Get an HTTP header.
    pub fn header(&self, key: &'static str) -> Option<&'_ str> {
        self.request.headers().get(key).map(|h| h.to_str().unwrap())
    }

    /// Set an HTTP header.
    pub fn set_header(mut self, key: &'static str, value: impl AsRef<str>) -> Self {
        let value = value.as_ref().to_owned();
        self.request
            .headers_mut()
            .insert(key, value.parse().unwrap());
        self
    }

    /// Get a local value.
    pub fn local<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.request.extensions().get()
    }

    /// Set a local value.
    pub fn set_local<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.request.extensions_mut().insert(val);
        self
    }

    ///  Access app-global state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Extract and parse a route parameter by name.
    ///
    /// Returns the results of parsing the parameter according to the inferred
    /// output type `T`.
    ///
    /// The name should *not* include the leading `:` or the trailing `*` (if
    /// any).
    ///
    /// # Errors
    ///
    /// Yields an `Err` if the parameter was found but failed to parse as an
    /// instance of type `T`.
    ///
    /// # Panics
    ///
    /// Panic if `key` is not a parameter for the route.
    pub fn param<T: FromStr>(&self, key: &str) -> Result<T, T::Err> {
        self.route_params.iter().rev().filter_map(|params| params.find(key)).next().unwrap().parse()
    }

    /// Reads the entire request body into a byte buffer.
    ///
    /// This method can be called after the body has already been read, but will
    /// produce an empty buffer.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    pub async fn body_bytes(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(1024);
        self.request.body_mut().read_to_end(&mut buf).await?;
        Ok(buf)
    }

    /// Reads the entire request body into a string.
    ///
    /// This method can be called after the body has already been read, but will
    /// produce an empty buffer.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    ///
    /// If the body cannot be interpreted as valid UTF-8, an `Err` is returned.
    pub async fn body_string(&mut self) -> std::io::Result<String> {
        let body_bytes = self.body_bytes().await?;
        Ok(String::from_utf8(body_bytes).map_err(|_| std::io::ErrorKind::InvalidData)?)
    }

    /// Reads and deserialized the entire request body via json.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    ///
    /// If the body cannot be interpreted as valid json for the target type `T`,
    /// an `Err` is returned.
    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> std::io::Result<T> {
        let body_bytes = self.body_bytes().await?;
        Ok(serde_json::from_slice(&body_bytes).map_err(|_| std::io::ErrorKind::InvalidData)?)
    }

    /// Get the URL querystring.
    pub fn query<'de, T: Deserialize<'de>>(&'de self) -> Result<T, crate::Error> {
        let query = self.uri().query();
        if query.is_none() {
            return Err(crate::Error::from(http::StatusCode::BAD_REQUEST));
        }
        Ok(serde_qs::from_str(query.unwrap())
            .map_err(|_| crate::Error::from(http::StatusCode::BAD_REQUEST))?)
    }

    /// Parse the request body as a form.
    pub async fn body_form<T: serde::de::DeserializeOwned>(&mut self) -> io::Result<T> {
        let body = self
            .body_bytes()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let res = serde_qs::from_bytes(&body).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("could not decode form: {}", e),
            )
        })?;
        Ok(res)
    }
}

impl<State> Read for Request<State> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut this = self.project();
        Pin::new(this.request.body_mut()).poll_read(cx, buf)
    }
}
