use futures::future::BoxFuture;
use tide_core::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// Middleware that injects a per-request [`slog::Logger`] onto the request [`Context`].
pub struct PerRequestLogger<State> {
    setup: Box<dyn (Fn(&mut Context<State>) -> slog::Logger) + Send + Sync + 'static>,
}

impl<State> PerRequestLogger<State> {
    /// Initialize this middleware with a function to create a per-request logger.
    ///
    /// # Examples
    ///
    /// ## Adding a base16 encoded per-request UUID onto the logger context
    ///
    /// ```
    /// use slog::o;
    ///
    /// let mut app = tide::new();
    ///
    /// let request_id = || uuid::Uuid::new_v4().to_simple().to_string();
    ///
    /// let root_logger = slog::Logger::root(slog::Discard, o!());
    /// app.middleware(tide_slog::PerRequestLogger::with_setup(move |_cx| root_logger.new(o! {
    ///     "request" => request_id(),
    /// })));
    /// ```
    ///
    /// ## Taking an externally provided request id from headers for the logger context
    ///
    /// ```
    /// use slog::o;
    ///
    /// let mut app = tide::new();
    ///
    /// let root_logger = slog::Logger::root(slog::Discard, o!());
    /// app.middleware(tide_slog::PerRequestLogger::with_setup(move |cx| root_logger.new(o! {
    ///     "request" => cx.headers().get("Request-Id").unwrap().to_str().unwrap().to_owned(),
    /// })));
    /// ```
    pub fn with_setup(
        setup: impl (Fn(&mut Context<State>) -> slog::Logger) + Send + Sync + 'static,
    ) -> Self {
        Self {
            setup: Box::new(setup),
        }
    }

    /// Initialize this middleware with a logger that will provided to each request.
    ///
    /// # Examples
    ///
    /// ```
    /// use slog::o;
    ///
    /// let mut app = tide::new();
    ///
    /// let root_logger = slog::Logger::root(slog::Discard, o!());
    /// app.middleware(tide_slog::PerRequestLogger::with_logger(root_logger));
    /// ```
    pub fn with_logger(logger: slog::Logger) -> Self {
        Self {
            setup: Box::new(move |_cx| logger.clone()),
        }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for PerRequestLogger<State> {
    fn handle<'a>(
        &'a self,
        mut cx: Context<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Response> {
        let logger = (self.setup)(&mut cx);
        cx.extensions_mut().insert(logger);
        next.run(cx)
    }
}

impl<State> std::fmt::Debug for PerRequestLogger<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerRequestLogger")
            .field("setup", &"[closure]")
            .finish()
    }
}
