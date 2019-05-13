use slog::{info, o, Drain};
use slog_async;
use slog_term;

use futures::future::BoxFuture;

use crate::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// Root logger for Tide. Wraps over logger provided by slog.SimpleLogger
#[derive(Debug)]
pub struct RootLogger {
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

impl Default for RootLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Stores information during request phase and logs information once the response
/// is generated.
impl<Data: Send + Sync + 'static> Middleware<Data> for RootLogger {
    fn handle<'a>(&'a self, cx: Context<Data>, next: Next<'a, Data>) -> BoxFuture<'a, Response> {
        box_async! {
            let path = cx.uri().path().to_owned();
            let method = cx.method().as_str().to_owned();

            let res = next.run(cx).await;
            let status = res.status();
            info!(self.inner_logger, "{} {} {}", method, path, status.as_str());
            res
        }
    }
}
