use std::sync::Arc;

use futures::future::FutureObj;

use crate::{endpoint::BoxedEndpoint, head::Head, Request, Response, RouteMatch};

mod default_headers;

pub use self::default_headers::DefaultHeaders;

/// Middleware for an app with state of type `Data`
pub trait ReqResMiddleware<Data>: Send + Sync {
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

/// Middleware that wraps around remaining middleware chain.
pub trait Middleware<Data: Clone + Send + Sync + 'static>: Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(&'a self, ctx: RequestContext<'a, Data>) -> FutureObj<'a, ResponseContext<Data>> {
        FutureObj::new(Box::new(ctx.next()))
    }
}

impl<T, Data: Clone + Send + Sync + 'static> Middleware<Data> for T
where
    T: ReqResMiddleware<Data>,
{
    fn handle<'a>(
        &'a self,
        mut ctx: RequestContext<'a, Data>,
    ) -> FutureObj<'a, ResponseContext<Data>> {
        FutureObj::new(Box::new(
            async move {
                if let Err(res) = await!(self.request(&mut ctx.app_data, &mut ctx.req, &ctx.params))
                {
                    return ctx.respond(res);
                }
                let mut ctx = await!(ctx.next());
                await!(self.response(&mut ctx.app_data, &ctx.head, &mut ctx.res));
                ctx
            },
        ))
    }
}

pub struct RequestContext<'a, Data> {
    pub app_data: Data,
    pub req: Request,
    pub params: RouteMatch<'a>,
    pub(crate) endpoint: &'a BoxedEndpoint<Data>,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware<Data> + Send + Sync>],
}

impl<'a, Data: Clone + Send + Sync + 'static> RequestContext<'a, Data> {
    /// Consume this context, and run remaining middleware chain to completion.
    pub fn next(mut self) -> FutureObj<'a, ResponseContext<Data>> {
        FutureObj::new(Box::new(
            async move {
                if let Some((current, next)) = self.next_middleware.split_first() {
                    self.next_middleware = next;
                    await!(current.handle(self))
                } else {
                    let (head, res) =
                        await!(self
                            .endpoint
                            .call(self.app_data.clone(), self.req, self.params));
                    ResponseContext {
                        app_data: self.app_data,
                        res,
                        head,
                    }
                }
            },
        ))
    }

    /// Consume this context, and respond with given `Response`.
    pub fn respond(self, res: Response) -> ResponseContext<Data> {
        ResponseContext::from_request_response(self, res)
    }
}

pub struct ResponseContext<Data> {
    pub app_data: Data,
    pub res: Response,
    pub head: Head,
}

impl<Data> ResponseContext<Data> {
    /// Build a new `ResponseContext` from `RequestContext` and `Response`.
    pub fn from_request_response(ctx: RequestContext<Data>, res: Response) -> Self {
        ResponseContext {
            app_data: ctx.app_data,
            res,
            head: ctx.req.into_parts().0.into(),
        }
    }
}
