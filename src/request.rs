use futures::future;
use std::ops::{Deref, DerefMut};

use crate::{body::Body, Extract, Response, RouteMatch};

/// An HTTP request.
///
/// A convenient alias for the `http::Request` type, using Tide's `Body`.
pub type Request = http::Request<Body>;

/// A value that can be computed on-demand from a request.
pub trait Compute: 'static + Sync + Send + Clone + Sized {
    /// Compute the value directly from the given request.
    fn compute_fresh(req: &mut Request) -> Self;

    /// Compute the value, or return a copy if it has already been computed for this request.
    fn compute(req: &mut Request) -> Self {
        if req.extensions().get::<ComputedMarker<Self>>().is_none() {
            let t = Self::compute_fresh(req);
            req.extensions_mut().insert(ComputedMarker(t));
        }

        req.extensions()
            .get::<ComputedMarker<Self>>()
            .unwrap()
            .0
            .clone()
    }
}

/// A private marker to ensure that computed values cannot be accessed directly through `extensions`
struct ComputedMarker<T>(T);

/// An extractor for computed values.
///
/// Endpoints can use this extractor to automatically compute values for a request, and re-use cached
/// results if those values have been computed previously (e.g. in some middleware).
#[derive(Clone)]
pub struct Computed<T>(T);

impl<T> Deref for Computed<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Computed<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<Data: 'static, T: Compute> Extract<Data> for Computed<T> {
    type Fut = future::Ready<Result<Self, Response>>;
    fn extract(data: &mut Data, req: &mut Request, params: &RouteMatch<'_>) -> Self::Fut {
        future::ok(Computed(T::compute(req)))
    }
}
