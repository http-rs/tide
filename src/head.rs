//! Types related to the headers (and metadata) of a request.
//!
//! This module includes extractors like `Path` that endpoints can use to
//! automatically parse out information from a request.

use futures::future;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use crate::{configuration::Store, Extract, ExtractSeed, IntoResponse, Request, Response, RouteMatch};

/// Header and metadata for a request.
///
/// Essentially an immutable, cheaply clonable version of `http::request::Parts`.
#[derive(Clone)]
pub struct Head {
    inner: Arc<http::request::Parts>,
}

impl From<http::request::Parts> for Head {
    fn from(parts: http::request::Parts) -> Self {
        Self {
            inner: Arc::new(parts),
        }
    }
}

impl Head {
    /// The full URI for this request
    pub fn uri(&self) -> &http::Uri {
        &self.inner.uri
    }

    /// The path portion of this request
    pub fn path(&self) -> &str {
        self.uri().path()
    }

    /// The query portion of this request
    pub fn query(&self) -> Option<&str> {
        self.uri().query()
    }

    /// The HTTP method being invoked
    pub fn method(&self) -> &http::Method {
        &self.inner.method
    }

    /// The HTTP headers
    pub fn headers(&self) -> &http::header::HeaderMap<http::header::HeaderValue> {
        &self.inner.headers
    }
}

pub struct NamedHeader(pub http::header::HeaderName);

pub struct Header<T>(pub T);

impl From<http::header::HeaderName> for NamedHeader {
    fn from(name: http::header::HeaderName) -> Self {
        NamedHeader(name)
    }
}

impl<T: From<http::header::HeaderValue> + Send + 'static, S: 'static> ExtractSeed<Header<T>, S> for NamedHeader {
    type Fut = future::Ready<Result<Header<T>, Response>>;
    fn extract(&self,
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let header = req.headers().get(&self.0);
        match header {
            Some(value) => future::ok(Header(value.clone().into())),
            None => future::err(http::status::StatusCode::BAD_REQUEST.into_response()),
        }
    }
}

impl<T: From<http::header::HeaderValue> + Send + 'static, S: 'static> ExtractSeed<Option<Header<T>>, S> for NamedHeader {
    type Fut = future::Ready<Result<Option<Header<T>>, Response>>;
    fn extract(&self,
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let header = req.headers().get(&self.0);
        match header {
            Some(value) => future::ok(Some(Header(value.clone().into()))),
            None => future::ok(None),
        }
    }
}


/// An extractor for path segments.
///
/// Routes can use wildcard path segments (`{}`), which are then extracted by the endpoint using
/// this `Path` extractor. Each `Path<T>` argument to an extractor parses the next wildcard segment
/// as type `T`, failing with a `NOT_FOUND` response if the segment fails to parse.
///
/// # Examples
///
/// Extracting a path segment with:
///
/// ```rust, no_run
/// # #![feature(async_await, futures_api)]
/// use tide::head;
///
/// async fn path_segment(head::Path(s): head::Path<String>) -> String {
///     println!("read segment: {}", s);
///     s
/// }
///
/// fn main() {
///     let mut app = tide::App::new(());
///     app.at("/path/{}").get(path_segment);
///     app.serve()
/// }
/// ```
///
pub struct Path<T>(pub T);

impl<T> Deref for Path<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Path<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// A key for storing the current segment match in a request's `extensions`
struct PathIdx(usize);

impl<T: Send + 'static + std::str::FromStr, S: 'static> Extract<S> for Path<T> {
    type Fut = future::Ready<Result<Self, Response>>;
    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let &PathIdx(i) = req.extensions().get::<PathIdx>().unwrap_or(&PathIdx(0));
        req.extensions_mut().insert(PathIdx(i + 1));
        match params {
            Some(params) => match params.vec[i].parse() {
                Ok(t) => future::ok(Path(t)),
                Err(_) => future::err(http::status::StatusCode::BAD_REQUEST.into_response()),
            },
            None => future::err(http::status::StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        }
    }
}

/// A trait providing the name of a named url segment
pub trait NamedSegment: Send + 'static + std::str::FromStr {
    const NAME: &'static str;
}

/// An extractor for named path segments
///
/// Allows routes to access named path segments (`{foo}`). Each `Named<T>` extracts a single
/// segment. `T` must implement the `NamedSegment` trait - to provide the segment name - and the
/// FromStr trait. Fails with a `BAD_REQUEST` response if the segment is not found, fails to
/// parse or if multiple identically named segments exist.
///
/// # Examples
///
/// Extracting a `Number` from a named path segment with:
///
/// ```rust, no_run
/// # #![feature(async_await, futures_api)]
/// use tide::head;
/// use tide::head::{Named, NamedSegment};
///
/// struct Number(i32);
///
/// impl NamedSegment for Number {
///     const NAME: &'static str = "num";
/// }
///
/// impl std::str::FromStr for Number {
///     type Err = std::num::ParseIntError;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         s.parse().map(|num| Number(num))
///     }
/// }
///
/// async fn named_segments(Named(number): Named<Number>) -> String {
///     let Number(num) = number;
///     format!("number: {}", num)
/// }
///
/// fn main() {
///     let mut app = tide::App::new(());
///     app.at("/path_named/{num}").get(named_segments);
///     app.serve()
/// }
/// ```
///
pub struct Named<T>(pub T);

impl<T: NamedSegment> Deref for Named<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: NamedSegment> DerefMut for Named<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: NamedSegment, S: 'static> Extract<S> for Named<T> {
    type Fut = future::Ready<Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        match params {
            Some(params) => params
                .map
                .get(T::NAME)
                .and_then(|segment| segment.parse().ok())
                .map_or(
                    future::err(http::status::StatusCode::BAD_REQUEST.into_response()),
                    |t| future::ok(Named(t)),
                ),
            None => future::err(http::status::StatusCode::BAD_REQUEST.into_response()),
        }
    }
}

/// A seed extracting a particular segment.
///
/// This extracts any `Named<T>` where `T: std::str::FromStr` by looking up the particular segment.
pub struct SegmentName(pub Cow<'static, str>);

impl<T: std::str::FromStr + Send + 'static, S: 'static> ExtractSeed<Named<T>, S> for SegmentName {
    type Fut = future::Ready<Result<Named<T>, Response>>;

    fn extract(&self,
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        match params {
            Some(params) => params
                .map
                .get(self.0.as_ref())
                .and_then(|segment| segment.parse().ok())
                .map_or(
                    future::err(http::status::StatusCode::BAD_REQUEST.into_response()),
                    |t| future::ok(Named(t)),
                ),
            None => future::err(http::status::StatusCode::BAD_REQUEST.into_response()),
        }
    }
}

/// An extractor for query string in URL
///
pub struct UrlQuery<T>(pub T);

impl<S, T> Extract<S> for UrlQuery<T>
where
    T: Send + std::str::FromStr + 'static,
    S: 'static,
{
    type Fut = future::Ready<Result<Self, Response>>;
    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        req.uri().query().and_then(|q| q.parse().ok()).map_or(
            future::err(http::status::StatusCode::BAD_REQUEST.into_response()),
            |q| future::ok(UrlQuery(q)),
        )
    }
}
