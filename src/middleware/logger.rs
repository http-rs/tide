use futures::future::BoxFuture;
use crate::{
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
/// app.middleware(tide::middleware::RequestLogger::new());
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
    ) -> Response {
        let path = ctx.uri().path().to_owned();
        let method = ctx.method().as_str().to_owned();
        log::trace!("IN => {} {}", method, path);
        let start = std::time::Instant::now();
        let res = next.run(ctx).await;
        let status = res.status();
        log::info!(
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
        Box::pin(async move { self.log_basic(ctx, next).await })
    }
}
