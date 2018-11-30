use slog::{info, o, Drain};
use slog_async;
use slog_term;

use futures::future::FutureObj;

use crate::{middleware::RequestContext, typemap::STATE, Middleware, Response};

/// Root logger for Tide. Wraps over logger provided by slog.SimpleLogger
///
/// Only used internally for now.
pub(crate) struct RootLogger;

impl RootLogger {
    pub fn init() -> RootLogger {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let log = slog::Logger::root(drain, o!());

        let logger = RootLogger {};

        // Store global logger instance.
        STATE.write().unwrap().insert(log);

        logger
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

                let map = STATE.read().unwrap();

                // Get global logger instance
                let logger = map.get::<slog::Logger>().unwrap();

                info!(logger, "{} {} {}", method, path, status.as_str());
                res
            },
        ))
    }
}
