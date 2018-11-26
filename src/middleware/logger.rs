use slog::{info, o, Drain};
use slog_async;
use slog_term;

use futures::future::FutureObj;

use crate::{middleware::RequestContext, Middleware, Response};

/// Root logger for Tide. Wraps over logger provided by slog.SimpleLogger
///
/// Only used internally for now.
pub(crate) struct RootLogger {
    // drain: dyn slog::Drain,
    inner_logger: slog::Logger,
}

impl RootLogger {
    pub fn new() -> RootLogger {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let log = slog::Logger::root(drain, o!());
        RootLogger { inner_logger: log }
    }
}

/// Stores information during request phase and logs information once the response
/// is generated.
impl<Data: Clone + Send> Middleware<Data> for RootLogger {
    fn handle<'a>(&'a self, ctx: RequestContext<'a, Data>) -> FutureObj<'a, Response> {
        FutureObj::new(Box::new(
            async move {
                let path = ctx.req.uri().path().to_owned();
                let method = ctx.req.method().as_str().to_owned();

                let res = await!(ctx.next());
                let status = res.status();
                info!(self.inner_logger, "{} {} {}", method, path, status.as_str());
                res
            },
        ))
    }
}
