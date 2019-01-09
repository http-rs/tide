use std::any::Any;
use std::sync::Arc;

use futures::future::FutureObj;

use crate::{router::EndpointData, Request, Response, RouteMatch};

mod default_headers;
pub mod logger;

pub use self::default_headers::DefaultHeaders;

/// Middleware that wraps around remaining middleware chain.
pub trait Middleware<Data>: Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(&'a self, ctx: RequestContext<'a, Data>) -> FutureObj<'a, Response>;
}

impl<Data, F> Middleware<Data> for F
where
    F: Send + Sync + Fn(RequestContext<Data>) -> FutureObj<Response>,
{
    fn handle<'a>(&'a self, ctx: RequestContext<'a, Data>) -> FutureObj<'a, Response> {
        (self)(ctx)
    }
}

pub struct RequestContext<'a, Data> {
    pub app_data: Data,
    pub req: Request,
    pub params: Option<RouteMatch<'a>>,
    pub(crate) endpoint: &'a EndpointData<Data>,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware<Data> + Send + Sync>],
}

impl<'a, Data: Clone + Send> RequestContext<'a, Data> {
    /// Get a configuration item of given type from the endpoint.
    pub fn get_item<T: Any + Clone + Send + Sync>(&self) -> Option<&T> {
        self.endpoint.store.read::<T>()
    }

    /// Consume this context, and run remaining middleware chain to completion.
    pub fn next(mut self) -> FutureObj<'a, Response> {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(self)
        } else {
            FutureObj::new(Box::new(self.endpoint.endpoint.call(
                self.app_data.clone(),
                self.req,
                self.params,
                &self.endpoint.store,
            )))
        }
    }
}
