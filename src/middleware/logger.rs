use crate::{
    middleware::{Middleware, Next},
    Request, Response,
};
use futures_core::future::BoxFuture;

/// A simple requests logger
///
/// # Examples
///
/// ```rust
///
/// let mut app = tide::Server::new();
/// app.middleware(tide::middleware::RequestLogger::new());
/// ```
#[derive(Debug, Clone, Default)]
pub struct RequestLogger;

impl RequestLogger {
    pub fn new() -> Self {
        Self::default()
    }

    async fn log_basic<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> crate::Result<Response> {
        let path = ctx.uri().path().to_owned();
        let method = ctx.method().to_string();
        log::trace!("IN => {} {}", method, path);
        let start = std::time::Instant::now();
        let res = next.run(ctx).await?;
        let status = res.status();
        log::info!(
            "{} {} {} {}ms",
            method,
            path,
            status,
            start.elapsed().as_millis()
        );
        Ok(res)
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for RequestLogger {
    fn handle<'a>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result<Response>> {
        Box::pin(async move { self.log_basic(ctx, next).await })
    }
}
