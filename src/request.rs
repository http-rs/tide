use async_std::io::{self, prelude::*};
use async_std::task::{Context, Poll};
use route_recognizer::Params;

use std::ops::Index;
use std::pin::Pin;
use std::{fmt, str::FromStr};

use crate::cookies::CookieData;
use crate::http::cookies::Cookie;
use crate::http::headers::{self, HeaderName, HeaderValues, ToHeaderValues};
use crate::http::{self, Body, Method, Mime, StatusCode, Url, Version};
use crate::Response;

/// An HTTP request.
///
/// The `Request` gives endpoints access to basic information about the incoming
/// request, route parameters, and various ways of accessing the request's body.
///
/// Requests also provide *extensions*, a type map primarily used for low-level
/// communication between middleware and endpoints.
#[derive(Debug)]
pub struct Request {
    pub(crate) req: http::Request,
    pub(crate) route_params: Vec<Params>,
}

#[derive(Debug)]
pub enum ParamError<E> {
    NotFound(String),
    ParsingError(E),
}

impl<E: fmt::Debug + fmt::Display> fmt::Display for ParamError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::NotFound(name) => write!(f, "Param \"{}\" not found!", name),
            ParamError::ParsingError(err) => write!(f, "Param failed to parse: {}", err),
        }
    }
}

impl<T: fmt::Debug + fmt::Display> std::error::Error for ParamError<T> {}

impl Request {
    /// Create a new `Request`.
    pub(crate) fn new(
        req: http_types::Request,
        route_params: Vec<Params>,
    ) -> Self {
        Self {
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

    /// Get the request content type as a `Mime`.
    ///
    /// This gets the request `Content-Type` header.
    ///
    /// [Read more on MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types)
    #[must_use]
    pub fn content_type(&self) -> Option<Mime> {
        self.req.content_type()
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
    /// Yields a `ParamError::ParsingError` if the parameter was found but failed to parse as an
    /// instance of type `T`.
    ///
    /// Yields a `ParamError::NotFound` if `key` is not a parameter for the route.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use tide::{Request, Result};
    ///
    /// async fn greet(req: Request<()>) -> Result<String> {
    ///     let name = req.param("name").unwrap_or("world".to_owned());
    ///     Ok(format!("Hello, {}!", name))
    /// }
    ///
    /// let mut app = tide::new();
    /// app.at("/hello").get(greet);
    /// app.at("/hello/:name").get(greet);
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) })}
    /// ```
    pub fn param<T: FromStr>(&self, key: &str) -> Result<T, ParamError<T::Err>> {
        self.route_params
            .iter()
            .rev()
            .find_map(|params| params.find(key))
            .ok_or_else(|| ParamError::NotFound(key.to_string()))
            .and_then(|param| param.parse().map_err(ParamError::ParsingError))
    }

    /// Parse the URL query component into a struct, using [serde_qs](serde_qs). To get the entire
    /// query as an unparsed string, use `request.url().query()`
    ///
    /// ```rust
    /// # fn main() -> Result<(), std::io::Error> { async_std::task::block_on(async {
    /// use tide::prelude::*;
    /// let mut app = tide::new();
    ///
    /// #[derive(Deserialize)]
    /// #[serde(default)]
    /// struct Page {
    ///     size: u8,
    ///     offset: u8,
    /// }
    /// impl Default for Page {
    ///     fn default() -> Self {
    ///         Self {
    ///             size: 25,
    ///             offset: 0,
    ///         }
    ///     }
    /// }
    /// app.at("/pages").post(|req: tide::Request<()>| async move {
    ///     let page: Page = req.query()?;
    ///     Ok(format!("page {}, with {} items", page.offset, page.size))
    /// });
    ///
    /// # if false {
    /// app.listen("localhost:8000").await?;
    /// # }
    ///
    /// // $ curl localhost:8000/pages
    /// // page 0, with 25 items
    ///
    /// // $ curl localhost:8000/pages?offset=1
    /// // page 1, with 25 items
    ///
    /// // $ curl localhost:8000/pages?offset=2&size=50
    /// // page 2, with 50 items
    ///
    /// // $ curl localhost:8000/pages?size=5000
    /// // failed with reason: number too large to fit in target type
    ///
    /// // $ curl localhost:8000/pages?size=all
    /// // failed with reason: invalid digit found in string
    /// # Ok(()) })}
    /// ```

    pub fn query<T: serde::de::DeserializeOwned>(&self) -> crate::Result<T> {
        self.req.query()
    }

    /// Set the body reader.
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.req.set_body(body)
    }

    /// Take the request body as a `Body`.
    ///
    /// This method can be called after the body has already been taken or read,
    /// but will return an empty `Body`.
    ///
    /// This is useful for consuming the body via an AsyncReader or AsyncBufReader.
    pub fn take_body(&mut self) -> Body {
        self.req.take_body()
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
    ///
    /// ```rust
    /// # fn main() -> Result<(), std::io::Error> { async_std::task::block_on(async {
    /// use tide::prelude::*;
    /// let mut app = tide::new();
    ///
    /// #[derive(Deserialize)]
    /// struct Animal {
    ///   name: String,
    ///   legs: u8
    /// }
    ///
    /// app.at("/").post(|mut req: tide::Request<()>| async move {
    ///     let animal: Animal = req.body_form().await?;
    ///     Ok(format!(
    ///         "hello, {}! i've put in an order for {} shoes",
    ///         animal.name, animal.legs
    ///     ))
    /// });
    ///
    /// # if false {
    /// app.listen("localhost:8000").await?;
    /// # }
    ///
    /// // $ curl localhost:8000/orders/shoes -d "name=chashu&legs=4"
    /// // hello, chashu! i've put in an order for 4 shoes
    ///
    /// // $ curl localhost:8000/orders/shoes -d "name=mary%20millipede&legs=750"
    /// // number too large to fit in target type
    /// # Ok(()) })}
    /// ```
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

impl AsRef<http::Request> for Request {
    fn as_ref(&self) -> &http::Request {
        &self.req
    }
}

impl AsMut<http::Request> for Request {
    fn as_mut(&mut self) -> &mut http::Request {
        &mut self.req
    }
}

impl AsRef<http::Headers> for Request {
    fn as_ref(&self) -> &http::Headers {
        self.req.as_ref()
    }
}

impl AsMut<http::Headers> for Request {
    fn as_mut(&mut self) -> &mut http::Headers {
        self.req.as_mut()
    }
}

impl Read for Request {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.req).poll_read(cx, buf)
    }
}

impl Into<http::Request> for Request {
    fn into(self) -> http::Request {
        self.req
    }
}

// NOTE: From cannot be implemented for this conversion because `State` needs to
// be constrained by a type.
impl Into<Response> for Request {
    fn into(mut self) -> Response {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(self.take_body());
        res
    }
}

impl IntoIterator for Request {
    type Item = (HeaderName, HeaderValues);
    type IntoIter = http_types::headers::IntoIter;

    /// Returns a iterator of references over the remaining items.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.req.into_iter()
    }
}

impl<'a> IntoIterator for &'a Request {
    type Item = (&'a HeaderName, &'a HeaderValues);
    type IntoIter = http_types::headers::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.req.iter()
    }
}

impl<'a> IntoIterator for &'a mut Request {
    type Item = (&'a HeaderName, &'a mut HeaderValues);
    type IntoIter = http_types::headers::IterMut<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.req.iter_mut()
    }
}

impl Index<HeaderName> for Request {
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

impl Index<&str> for Request {
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
