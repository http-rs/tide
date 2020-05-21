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
#[derive(Debug, Clone)]
pub struct LogMiddleware {
    _priv: (),
}

impl LogMiddleware {
    /// Create a new instance of `LogMiddleware`.
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Log a request and a response.
    async fn log<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> crate::Result {
        let path = ctx.uri().path().to_owned();
        let method = ctx.method().to_string();
        log::info!("<-- Request received", {
            method: method,
            path: path,
        });
        let start = std::time::Instant::now();
        match next.run(ctx).await {
            Ok(res) => {
                let status = res.status();
                if status.is_server_error() {
                    log::error!("--> Response sent", {
                        method: method,
                        path: path,
                        status: status as u16,
                        duration: format!("{:?}", start.elapsed()),
                    });
                } else if status.is_client_error() {
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
                Ok(res)
            }
            Err(err) => {
                log::error!("{}", err.to_string(), {
                    method: method,
                    path: path,
                    status: err.status() as u16,
                    duration: format!("{:?}", start.elapsed()),
                });
                Err(err)
            }
        }
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
