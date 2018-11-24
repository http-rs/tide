use futures::future::FutureObj;

use crate::{head::Head, Request, Response, RouteMatch};

mod default_headers;

pub use self::default_headers::DefaultHeaders;

/// Middleware for an app with state of type `Data`
pub trait Middleware<Data>: Send + Sync {
    /// Asynchronously transform the incoming request, or abort further handling by immediately
    /// returning a response.
    fn request<'a>(
        &'a self,
        data: &'a mut Data,
        req: &'a mut Request,
        params: &'a RouteMatch<'_>,
    ) -> FutureObj<'a, Result<(), Response>> {
        FutureObj::new(Box::new(async { Ok(()) }))
    }

    /// Asynchronously transform the outgoing response.
    fn response<'a>(
        &'a self,
        data: &'a mut Data,
        head: &'a Head,
        resp: &'a mut Response,
    ) -> FutureObj<'a, ()> {
        FutureObj::new(Box::new(async {}))
    }

    // TODO: provide the following, intended to fire *after* the body has been fully sent

    /*
    fn finish(
        &self,
        data: &mut Data,
        head: &Head,
        resp: &Response,
    ) -> FutureObj<'static, ()>;
    */
}
