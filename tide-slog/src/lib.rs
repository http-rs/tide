//! Crate that provides helpers and/or middlewares for Tide
//! related to structured logging with slog.

#![feature(async_await)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

use slog::{info, o, trace, Drain};
use slog_async;
use slog_term;

use futures::future::BoxFuture;
use futures::prelude::*;

use tide_core::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// RequestLogger based on slog.SimpleLogger
#[derive(Debug)]
pub struct RequestLogger {
    // drain: dyn slog::Drain,
    inner: slog::Logger,
}

impl RequestLogger {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_logger(logger: slog::Logger) -> Self {
        Self { inner: logger }
    }
}

impl Default for RequestLogger {
    fn default() -> Self {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let log = slog::Logger::root(drain, o!());
        Self { inner: log }
    }
}

/// Stores information during request phase and logs information once the response
/// is generated.
impl<State: Send + Sync + 'static> Middleware<State> for RequestLogger {
    fn handle<'a>(&'a self, cx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        FutureExt::boxed(async move {
            let path = cx.uri().path().to_owned();
            let method = cx.method().as_str().to_owned();
            trace!(self.inner, "IN => {} {}", method, path);
            let start = std::time::Instant::now();
            let res = next.run(cx).await;
            let status = res.status();
            info!(
                self.inner,
                "{} {} {} {}ms",
                method,
                path,
                status.as_str(),
                start.elapsed().as_millis()
            );
            res
        })
    }
}
