//! Types related to the headers (and metadata) of a request.
//!
//! This module includes extractors like `Path` that endpoints can use to
//! automatically parse out information from a request.

use futures::future;
use std::sync::Arc;

use crate::{Extract, IntoResponse, Request, Response, RouteMatch};

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
}

/// An extractor for path components.
///
/// Routes can use wildcard path components (`{}`), which are then extracted by the endpoint using
/// this `Path` extractor. Each `Path<T>` argument to an extractor parses the next wildcard component
/// as type `T`, failing with a `NOT_FOUND` response if the component fails to parse.
pub struct Path<T>(pub T);

/// A key for storing the current component match in a request's `extensions`
struct PathIdx(usize);

impl<T: Send + 'static + std::str::FromStr, S: 'static> Extract<S> for Path<T> {
    type Fut = future::Ready<Result<Self, Response>>;
    fn extract(data: &mut S, req: &mut Request, params: &RouteMatch<'_>) -> Self::Fut {
        let &PathIdx(i) = req.extensions().get::<PathIdx>().unwrap_or(&PathIdx(0));
        req.extensions_mut().insert(PathIdx(i + 1));
        match params.vec[i].parse() {
            Ok(t) => future::ok(Path(t)),
            Err(_) => future::err(http::status::StatusCode::BAD_REQUEST.into_response()),
        }
    }
}
