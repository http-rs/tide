use crate::log;
use crate::utils::TideState;
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
    async fn log<'a, State: TideState>(
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
        if status.is_server_error() {
            if let Some(error) = response.error() {
                log::error!("Internal error --> Response sent", {
                    message: error.to_string(),
                    method: method,
                    path: path,
                    status: status as u16,
                    duration: format!("{:?}", start.elapsed()),
                });
            } else {
                log::error!("Internal error --> Response sent", {
                    method: method,
                    path: path,
                    status: status as u16,
                    duration: format!("{:?}", start.elapsed()),
                });
            }
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
        Ok(response)
    }
}

#[async_trait::async_trait]
impl<State: TideState> Middleware<State> for LogMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> crate::Result {
        self.log(req, next).await
    }
}
