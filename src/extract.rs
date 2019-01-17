use futures::prelude::*;

use crate::{configuration::Store, Request, Response, RouteMatch};

/// An extractor for an app with `Data`
pub trait Extract<Data>: Send + Sized + 'static {
    /// The async result of `extract`.
    ///
    /// The `Err` case represents that the endpoint should not be invoked, but
    /// rather the given response should be returned immediately.
    type Fut: Future<Output = Result<Self, Response>> + Send + 'static;

    /// Attempt to extract a value from the given request.
    fn extract(
        data: &mut Data,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut;
}
