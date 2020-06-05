use crate::log;
use crate::utils::BoxFuture;
use crate::{Middleware, Next, Request};

/// Log all incoming requests and responses.
///
/// This middleware is enabled by default in Tide.
///
/// # Examples
///
/// ```
/// let mut app = tide::Server::new();
/// app.middleware(tide::log::LogMiddleware::new());
/// ```
#[derive(Debug, Default, Clone)]
pub struct LogMiddleware {
    _priv: (),
}

impl LogMiddleware {
    /// Create a new instance of `LogMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Log a request and a response.
    async fn log<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> crate::Result {
        let path = ctx.url().path().to_owned();
        let method = ctx.method().to_string();
        log::info!("<-- Request received", {
            method: method,
            path: path,
        });
        let start = std::time::Instant::now();
        let response = next.run(ctx).await;
        let status = response.status();
        if let Some(error) = response.error() {
            log::error!("--> Response error", {
                message: error.to_string(),
                method: method,
                path: path,
                status: status as u16,
                duration: format!("{:?}", start.elapsed()),
            });
        } else if status.is_client_error() || status.is_server_error() {
            log::warn!("--> Response sent", {
                method: method,
                path: path,
                status: status as u16,
                duration: format!("{:?}", start.elapsed()),
            });
        } else {
            log::info!("--> Response sent", {
                method: method,
                path: path,
                status: status as u16,
                duration: format!("{:?}", start.elapsed()),
            });
        }
        Ok(response)
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for LogMiddleware {
    fn handle<'a>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result> {
        Box::pin(async move { self.log(ctx, next).await })
    }
}
