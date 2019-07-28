use crate::ContextExt;
use futures::future::BoxFuture;
use slog::{info, trace};
use tide_core::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// Middleware that logs minimal request details to the current request's [`slog::Logger`].
///
/// Relies on having a [`PerRequestLogger`][crate::PerRequestLogger] middleware instance setup
/// beforehand to get the logger from.
///
/// # Examples
///
/// ```
/// use slog::o;
///
/// let mut app = tide::Server::new();
///
/// let root_logger = slog::Logger::root(slog::Discard, o!());
/// app.middleware(tide_slog::PerRequestLogger::with_logger(root_logger));
/// app.middleware(tide_slog::RequestLogger::new());
/// ```
#[derive(Debug)]
pub struct RequestLogger {
    // In case we want to make this configurable in the future
    _reserved: (),
}

impl RequestLogger {
    /// Create a new [`RequestLogger`] instance.
    pub fn new() -> Self {
        Self { _reserved: () }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for RequestLogger {
    fn handle<'a>(&'a self, cx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let logger = cx.logger().clone();
            let path = cx.uri().path().to_owned();
            let method = cx.method().as_str().to_owned();

            trace!(
                logger,
                "IN => {method} {path}",
                method = &method,
                path = &path
            );

            let start = std::time::Instant::now();
            let res = next.run(cx).await;
            let status = res.status();

            info!(
                logger,
                "{method} {path} {status} {elapsed}ms",
                method = &method,
                path = &path,
                status = status.as_str(),
                elapsed = start.elapsed().as_millis(),
            );

            res
        })
    }
}

impl Default for RequestLogger {
    fn default() -> Self {
        Self::new()
    }
}
