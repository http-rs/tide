use std::sync::Arc;

use futures::future::FutureObj;

use crate::{endpoint::BoxedEndpoint, head::Head, Request, Response, RouteMatch};

mod default_headers;

pub use self::default_headers::DefaultHeaders;

/// Middleware that wraps around remaining middleware chain.
pub trait Middleware<Data>: Send + Sync {
    /// Asynchronously handle the request, and return a response.
    fn handle<'a>(&'a self, ctx: RequestContext<'a, Data>) -> FutureObj<'a, ResponseContext<Data>>;
}

impl<Data, F> Middleware<Data> for F
where
    F: Send + Sync + Fn(RequestContext<Data>) -> FutureObj<ResponseContext<Data>>,
{
    fn handle<'a>(&'a self, ctx: RequestContext<'a, Data>) -> FutureObj<'a, ResponseContext<Data>> {
        (self)(ctx)
    }
}

pub struct RequestContext<'a, Data> {
    pub app_data: Data,
    pub req: Request,
    pub params: RouteMatch<'a>,
    pub(crate) endpoint: &'a BoxedEndpoint<Data>,
    pub(crate) next_middleware: &'a [Arc<dyn Middleware<Data> + Send + Sync>],
}

impl<'a, Data> RequestContext<'a, Data> {
    /// Consume this context, and respond with given `Response`.
    pub fn respond(self, res: Response) -> ResponseContext<Data> {
        ResponseContext::from_request_response(self, res)
    }
}

impl<'a, Data: Clone + Send> RequestContext<'a, Data> {
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
