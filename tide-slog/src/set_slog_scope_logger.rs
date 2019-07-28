use crate::ContextExt;
use futures::future::{BoxFuture, FutureExt as _};
use slog_scope_futures::FutureExt as _;
use tide_core::{
    middleware::{Middleware, Next},
    Response,
};

#[cfg_attr(docrs, doc(cfg(feature = "scope")))]
/// Middleware that ensures the current request's [`slog::Logger`] will be accessible using
/// [`slog-scope::logger`] during all following processing of the request.
///
/// Relies on having a [`PerRequestLogger`][crate::PerRequestLogger] middleware instance setup
/// beforehand to get the logger from.
///
/// This can be used along with [`slog-stdlog`](https://docs.rs/slog-stdlog/) to
/// integrate per-request logging with middleware that use [`log`](https://docs.rs/log)`.
///
/// # Examples
///
/// ```
/// use slog::o;
///
/// let root_logger = slog::Logger::root(slog::Discard, o!());
///
/// let _guard = slog_scope::set_global_logger(root_logger.clone());
/// slog_stdlog::init()?;
///
/// let mut app = tide::new();
///
/// app.middleware(tide_slog::PerRequestLogger::with_logger(root_logger));
/// app.middleware(tide_slog::SetSlogScopeLogger);
///
/// // The default tide request logger uses `log`, but since we are using `slog-stdlog` and run
/// // `SetSlogScopeLogger` first it will be redirected into the per-request logger instance.
/// app.middleware(tide::middleware::RequestLogger::new());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
pub struct SetSlogScopeLogger;

impl<State: Send + Sync + 'static> Middleware<State> for SetSlogScopeLogger {
    fn handle<'a>(
        &'a self,
        cx: tide_core::Context<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Response> {
        let logger = cx.logger().clone();
        next.run(cx).with_logger(logger).boxed()
    }
}
