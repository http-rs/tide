use slog::{info, o, Drain};
use slog_async;
use slog_term;

use futures::future::FutureObj;

use crate::{head::Head, Middleware, Response};

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
impl<Data> Middleware<Data> for RootLogger {
    fn response(
        &self,
        data: &mut Data,
        head: &Head,
        resp: Response,
    ) -> FutureObj<'static, Response> {
        let status = resp.status();
        info!(
            self.inner_logger,
            "{} {} {}",
            head.method(),
            head.path(),
            status.as_str()
        );

        FutureObj::new(Box::new(async { resp }))
    }
}
