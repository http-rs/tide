//! Crate that provides helpers and/or middlewares for Tide
//! related to logging.

#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

use futures::future::BoxFuture;

use log::{info, trace};

use tide_core::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// A simple requests logger
///
/// # Examples
///
/// ```rust
///
/// let mut app = tide::App::new();
/// app.middleware(tide_log::RequestLogger::new());
/// ```
#[derive(Debug, Clone, Default)]
pub struct RequestLogger {
    target: String,
}

impl RequestLogger {
    /// Create a new instance of logger with default target as
    /// "requests"
    pub fn new() -> Self {
        Self {
            target: "requests".to_owned(),
        }
    }

    /// Create a new instance of logger with supplied `target` for
    /// logging.
    pub fn with_target(target: String) -> Self {
        Self { target }
    }

    async fn log_basic<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Context<State>,
        next: Next<'a, State>,
    ) -> Response {
        let path = ctx.uri().path().to_owned();
        let method = ctx.method().as_str().to_owned();
        trace!(target: &self.target, "IN => {} {}", method, path);
        let start = std::time::Instant::now();
        let res = next.run(ctx).await;
        let status = res.status();
        info!(
            target: &self.target,
            "{} {} {} {}ms",
            method,
            path,
            status.as_str(),
            start.elapsed().as_millis()
        );
        res
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for RequestLogger {
    fn handle<'a>(&'a self, ctx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        Box::pin(async move { self.log_basic(ctx, next).await })
    }
}
