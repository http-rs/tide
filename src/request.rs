use async_std::io::{self, prelude::*, BufReader};
use async_std::task::{Context, Poll};
use route_recognizer::Params;

use std::ops::Index;
use std::pin::Pin;
use std::{str::FromStr, sync::Arc};

use crate::cookies::CookieData;
use crate::http::cookies::Cookie;
use crate::http::headers::{self, HeaderName, HeaderValues, ToHeaderValues};
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
    pub(crate) req: http::Request,
    pub(crate) route_params: Vec<Params>,
}

impl<State> Request<State> {
    /// Create a new `Request`.
    pub(crate) fn new(
        state: Arc<State>,
        req: http_types::Request,
        route_params: Vec<Params>,
    ) -> Self {
        Self {
            state,
            req,
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
        self.req.method()
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
    ///     assert_eq!(req.url(), &"/".parse::<tide::http::Url>().unwrap());
    ///     Ok("")
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    #[must_use]
    pub fn url(&self) -> &Url {
        self.req.url()
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
        self.req.version()
    }

    /// Get the peer socket address for the underlying transport, if
    /// that information is available for this request.
    #[must_use]
    pub fn peer_addr(&self) -> Option<&str> {
        self.req.peer_addr()
    }

    /// Get the local socket address for the underlying transport, if
    /// that information is available for this request.
    #[must_use]
    pub fn local_addr(&self) -> Option<&str> {
        self.req.local_addr()
    }

    /// Get the remote address for this request.
    ///
    /// This is determined in the following priority:
    /// 1. `Forwarded` header `for` key
    /// 2. The first `X-Forwarded-For` header
    /// 3. Peer address of the transport
    #[must_use]
    pub fn remote(&self) -> Option<&str> {
        self.req.remote()
    }

    /// Get the destination host for this request.
    ///
    /// This is determined in the following priority:
    /// 1. `Forwarded` header `host` key
    /// 2. The first `X-Forwarded-Host` header
    /// 3. `Host` header
    /// 4. URL domain, if any
    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.req.host()
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
    ///     assert_eq!(req.header("X-Forwarded-For").unwrap(), "127.0.0.1");
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
        self.req.header(key)
    }

    /// Get a mutable reference to a header.
    pub fn header_mut(&mut self, name: impl Into<HeaderName>) -> Option<&mut HeaderValues> {
        self.req.header_mut(name)
    }

    /// Set an HTTP header.
    pub fn insert_header(
        &mut self,
        name: impl Into<HeaderName>,
        values: impl ToHeaderValues,
    ) -> Option<HeaderValues> {
        self.req.insert_header(name, values)
    }

    /// Append a header to the headers.
    ///
    /// Unlike `insert` this function will not override the contents of a header, but insert a
    /// header if there aren't any. Or else append to the existing list of headers.
    pub fn append_header(&mut self, name: impl Into<HeaderName>, values: impl ToHeaderValues) {
        self.req.append_header(name, values)
    }

    /// Remove a header.
    pub fn remove_header(&mut self, name: impl Into<HeaderName>) -> Option<HeaderValues> {
        self.req.remove_header(name)
    }

    /// An iterator visiting all header pairs in arbitrary order.
    #[must_use]
    pub fn iter(&self) -> headers::Iter<'_> {
        self.req.iter()
    }

    /// An iterator visiting all header pairs in arbitrary order, with mutable references to the
    /// values.
    #[must_use]
    pub fn iter_mut(&mut self) -> headers::IterMut<'_> {
        self.req.iter_mut()
    }

    /// An iterator visiting all header names in arbitrary order.
    #[must_use]
    pub fn header_names(&self) -> headers::Names<'_> {
        self.req.header_names()
    }

    /// An iterator visiting all header values in arbitrary order.
    #[must_use]
    pub fn header_values(&self) -> headers::Values<'_> {
        self.req.header_values()
    }

    /// Get a request extension value.
    #[must_use]
    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.req.ext().get()
    }

    /// Set a request extension value.
    pub fn set_ext<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.req.ext_mut().insert(val)
    }

    #[must_use]
    ///  Access application scoped state.
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

    /// Get the URL querystring.
    pub fn query<T: serde::de::DeserializeOwned>(&self) -> crate::Result<T> {
        // Default to an empty query string if no query parameter has been specified.
        // This allows successful deserialisation of structs where all fields are optional
        // when none of those fields has actually been passed by the caller.
        let query = self.url().query().unwrap_or("");
        serde_qs::from_str(query).map_err(|e| {
            // Return the displayable version of the deserialisation error to the caller
            // for easier debugging.
            crate::Error::from_str(StatusCode::BadRequest, format!("{}", e))
        })
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
    pub async fn body_bytes(&mut self) -> crate::Result<Vec<u8>> {
        let res = self.req.body_bytes().await?;
        Ok(res)
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
    pub async fn body_string(&mut self) -> crate::Result<String> {
        let res = self.req.body_string().await?;
        Ok(res)
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
    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> crate::Result<T> {
        let res = self.req.body_json().await?;
        Ok(res)
    }

    /// Parse the request body as a form.
    pub async fn body_form<T: serde::de::DeserializeOwned>(&mut self) -> crate::Result<T> {
        let res = self.req.body_form().await?;
        Ok(res)
    }

    /// returns a `Cookie` by name of the cookie.
    #[must_use]
    pub fn cookie(&self, name: &str) -> Option<Cookie<'static>> {
        self.ext::<CookieData>()
            .and_then(|cookie_data| cookie_data.content.read().unwrap().get(name).cloned())
    }

    /// Get the current content type
    #[must_use]
    pub fn content_type(&self) -> Option<http_types::Mime> {
        self.req.content_type()
    }

    /// Get the length of the body stream, if it has been set.
    ///
    /// This value is set when passing a fixed-size object into as the body. E.g. a string, or a
    /// buffer. Consumers of this API should check this value to decide whether to use `Chunked`
    /// encoding, or set the response length.
    #[must_use]
    pub fn len(&self) -> Option<usize> {
        self.req.len()
    }

    /// Returns `true` if the request has a set body stream length of zero, `false` otherwise.
    #[must_use]
    pub fn is_empty(&self) -> Option<bool> {
        Some(self.req.len()? == 0)
    }
}

impl<State> AsMut<http::Request> for Request<State> {
    fn as_mut(&mut self) -> &mut http::Request {
        &mut self.req
    }
}

impl<State> AsRef<http::Request> for Request<State> {
    fn as_ref(&self) -> &http::Request {
        &self.req
    }
}

impl<State> Read for Request<State> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.req).poll_read(cx, buf)
    }
}

impl<State> Into<http::Request> for Request<State> {
    fn into(self) -> http::Request {
        self.req
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
        self.req.into_iter()
    }
}

impl<'a, State> IntoIterator for &'a Request<State> {
    type Item = (&'a HeaderName, &'a HeaderValues);
    type IntoIter = http_types::headers::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.req.iter()
    }
}

impl<'a, State> IntoIterator for &'a mut Request<State> {
    type Item = (&'a HeaderName, &'a mut HeaderValues);
    type IntoIter = http_types::headers::IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.req.iter_mut()
    }
}

impl<State> Index<HeaderName> for Request<State> {
    type Output = HeaderValues;

    /// Returns a reference to the value corresponding to the supplied name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not present in `Request`.
    #[inline]
    fn index(&self, name: HeaderName) -> &HeaderValues {
        &self.req[name]
    }
}

impl<State> Index<&str> for Request<State> {
    type Output = HeaderValues;

    /// Returns a reference to the value corresponding to the supplied name.
    ///
    /// # Panics
    ///
    /// Panics if the name is not present in `Request`.
    #[inline]
    fn index(&self, name: &str) -> &HeaderValues {
        &self.req[name]
    }
}

pub(crate) fn rest(route_params: &[Params]) -> Option<&str> {
    route_params
        .last()
        .and_then(|params| params.find("--tide-path-rest"))
}
