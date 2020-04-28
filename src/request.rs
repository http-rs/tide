use route_recognizer::Params;
use serde::Deserialize;

use async_std::io::{self, prelude::*, BufReader};
use async_std::task::{Context, Poll};

use std::pin::Pin;
use std::{str::FromStr, sync::Arc};

use crate::cookies::CookieData;
use crate::http::cookies::Cookie;
use crate::http::headers::{HeaderName, HeaderValues};
use crate::http::{self, Method, StatusCode, Url, Version};
use crate::Response;

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
    pub(crate) request: http::Request,
    pub(crate) route_params: Vec<Params>,
}

impl<State> Request<State> {
    pub(crate) fn new(
        state: Arc<State>,
        request: http_types::Request,
        route_params: Vec<Params>,
    ) -> Self {
        Self {
            state,
            request,
            route_params,
        }
    }

    /// Access the request's HTTP method.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use tide::Request;
    ///
    /// let mut app = tide::new();
    /// app.at("/").get(|req: Request<()>| async move {
    ///     assert_eq!(req.method(), http_types::Method::Get);
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    #[must_use]
    pub fn method(&self) -> Method {
        self.request.method()
    }

    /// Access the request's full URI method.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use tide::Request;
    ///
    /// let mut app = tide::new();
    /// app.at("/").get(|req: Request<()>| async move {
    ///     assert_eq!(req.uri(), &"/".parse::<tide::http::Url>().unwrap());
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    #[must_use]
    pub fn uri(&self) -> &Url {
        self.request.url()
    }

    /// Access the request's HTTP version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use tide::Request;
    ///
    /// let mut app = tide::new();
    /// app.at("/").get(|req: Request<()>| async move {
    ///     assert_eq!(req.version(), Some(http_types::Version::Http1_1));
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    #[must_use]
    pub fn version(&self) -> Option<Version> {
        self.request.version()
    }

    /// Get an HTTP header.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use tide::Request;
    ///
    /// let mut app = tide::new();
    /// app.at("/").get(|req: Request<()>| async move {
    ///     assert_eq!(req.header("X-Forwarded-For").unwrap().last().as_str(), "127.0.0.1");
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    #[must_use]
    pub fn header(
        &self,
        key: impl Into<http_types::headers::HeaderName>,
    ) -> Option<&http_types::headers::HeaderValues> {
        self.request.header(key)
    }

    /// Get a request extension value.
    #[must_use]
    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.request.ext().get()
    }

    /// Set a request extension value.
    pub fn set_ext<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.request.ext_mut().insert(val);
        self
    }

    #[must_use]
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
        self.route_params
            .iter()
            .rev()
            .find_map(|params| params.find(key))
            .unwrap()
            .parse()
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use tide::Request;
    ///
    /// let mut app = tide::new();
    /// app.at("/").get(|mut req: Request<()>| async move {
    ///     let _body: Vec<u8> = req.body_bytes().await.unwrap();
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    pub async fn body_bytes(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(1024);
        self.request.read_to_end(&mut buf).await?;
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use tide::Request;
    ///
    /// let mut app = tide::new();
    /// app.at("/").get(|mut req: Request<()>| async move {
    ///     let _body: String = req.body_string().await.unwrap();
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
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
        // Default to an empty query string if no query parameter has been specified.
        // This allows successful deserialisation of structs where all fields are optional
        // when none of those fields has actually been passed by the caller.
        let query = self.uri().query().unwrap_or("");
        serde_qs::from_str(query).map_err(|e| {
            // Return the displayable version of the deserialisation error to the caller
            // for easier debugging.
            crate::Error::from_str(StatusCode::BadRequest, format!("{}", e))
        })
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

    /// returns a `Cookie` by name of the cookie.
    #[must_use]
    pub fn cookie(&self, name: &str) -> Option<Cookie<'static>> {
        if let Some(cookie_data) = self.local::<CookieData>() {
            cookie_data.content.read().unwrap().get(name).cloned()
        } else {
            None
        }
    }

    /// Get the length of the body.
    #[must_use]
    pub fn len(&self) -> Option<usize> {
        self.request.len()
    }
    /// Checks if the body is empty.
    #[must_use]
    pub fn is_empty(&self) -> Option<bool> {
        Some(self.request.len()? == 0)
    }
}

impl<State> AsMut<http::Request> for Request<State> {
    fn as_mut(&mut self) -> &mut http::Request {
        &mut self.request
    }
}

impl<State> AsRef<http::Request> for Request<State> {
    fn as_ref(&self) -> &http::Request {
        &self.request
    }
}

impl<State> Read for Request<State> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.request).poll_read(cx, buf)
    }
}

impl<State> Into<http::Request> for Request<State> {
    fn into(self) -> http::Request {
        self.request
    }
}

// NOTE: From cannot be implemented for this conversion because `State` needs to
// be constrained by a type.
impl<State: Send + Sync + 'static> Into<Response> for Request<State> {
    fn into(self) -> Response {
        Response::new(StatusCode::Ok).body(BufReader::new(self))
    }
}

impl<State> IntoIterator for Request<State> {
    type Item = (HeaderName, HeaderValues);
    type IntoIter = http_types::headers::IntoIter;

    /// Returns a iterator of references over the remaining items.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.request.into_iter()
    }
}

impl<'a, State> IntoIterator for &'a Request<State> {
    type Item = (&'a HeaderName, &'a HeaderValues);
    type IntoIter = http_types::headers::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.request.iter()
    }
}

impl<'a, State> IntoIterator for &'a mut Request<State> {
    type Item = (&'a HeaderName, &'a mut HeaderValues);
    type IntoIter = http_types::headers::IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.request.iter_mut()
    }
}

pub(crate) fn rest(route_params: &[Params]) -> Option<&str> {
    route_params
        .last()
        .and_then(|params| params.find("--tide-path-rest"))
}
