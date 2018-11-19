use slog::{info, o, Drain};
use slog_async;
use slog_term;

use futures::future::FutureObj;

use std::sync::RwLock;

use crate::{head::Head, Middleware, Request, Response, RouteMatch};

/// Root logger for Tide. Wraps over logger provided by slog.SimpleLogger
///
/// Only used internally for now.
struct RootLogger {
    // drain: dyn slog::Drain,
    inner_logger: slog::Logger,
}

impl RootLogger {
    fn new() -> RootLogger {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let log = slog::Logger::root(drain, o!());
        RootLogger { inner_logger: log }
    }
}

struct SimpleLogData {
    path: String,
    method: String,
}

/// General logger used in application middleware.SimpleLogger
///
/// Only internal to crate
pub(crate) struct SimpleLogger {
    data: RwLock<SimpleLogData>,
    logger: RootLogger,
}

impl SimpleLogger {
    pub fn new() -> Self {
        SimpleLogger {
            data: RwLock::new(SimpleLogData {
                path: String::new(),
                method: String::new(),
            }),
            logger: RootLogger::new(),
        }
    }
}

/// Stores information during request phase and logs information once the response
/// is generated.
impl<Data> Middleware<Data> for SimpleLogger {
    fn request(
        &self,
        data: &mut Data,
        req: Request,
        params: &RouteMatch<'_>,
    ) -> FutureObj<'static, Result<Request, Response>> {
        let mut data = self.data.write().unwrap();
        data.path = req.uri().path().to_owned();
        data.method = req.method().as_str().to_owned();

        FutureObj::new(Box::new(async { Ok(req) }))
    }

    fn response(
        &self,
        data: &mut Data,
        head: &Head,
        resp: Response,
    ) -> FutureObj<'static, Response> {
        let status = resp.status();
        let data = self.data.read().unwrap();
        info!(
            self.logger.inner_logger,
            "{} {} {}",
            data.method.clone(),
            data.path.clone(),
            status.as_str()
        );

        FutureObj::new(Box::new(async { resp }))
    }
}
