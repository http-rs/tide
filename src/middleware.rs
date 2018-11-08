use futures::future::FutureObj;

use crate::{head::Head, Request, Response, RouteMatch};

/// Middleware for an app with state of type `Data`
pub trait Middleware<Data>: Send + Sync {
    /// Asynchronously transform the incoming request, or abort further handling by immediately
    /// returning a response.
    fn request(
        &self,
        data: &mut Data,
        req: Request,
        params: &RouteMatch<'_>,
    ) -> FutureObj<'static, Result<Request, Response>>;

    /// Asynchronously transform the outgoing response.
    fn response(
        &self,
        data: &mut Data,
        head: &Head,
        resp: Response,
    ) -> FutureObj<'static, Response>;

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
