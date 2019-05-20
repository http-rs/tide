#![feature(async_await)]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

use futures::future::BoxFuture;
use log::{info, trace};
use tide::{
    middleware::{Middleware, Next},
    Context, Response,
};

macro_rules! box_async {
    {$($t:tt)*} => {
        ::futures::future::FutureExt::boxed(async move { $($t)* })
    };
}

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
pub struct RequestLogger;

impl RequestLogger {
    pub fn new() -> Self {
        Self::default()
    }

    async fn log_basic<'a, Data: Send + Sync + 'static>(
        &'a self,
        ctx: Context<Data>,
        next: Next<'a, Data>,
    ) -> tide::Response {
        let path = ctx.uri().path().to_owned();
        let method = ctx.method().as_str().to_owned();
        trace!("IN => {} {}", method, path);
        let start = std::time::Instant::now();
        let res = next.run(ctx).await;
        let status = res.status();
        info!(
            "{} {} {} {}ms",
            method,
            path,
            status.as_str(),
            start.elapsed().as_millis()
        );
        res
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for RequestLogger {
    fn handle<'a>(&'a self, ctx: Context<Data>, next: Next<'a, Data>) -> BoxFuture<'a, Response> {
        box_async! { self.log_basic(ctx, next).await }
    }
}
