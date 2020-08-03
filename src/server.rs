//! An HTTP server

use async_std::io;
use async_std::sync::Arc;

use crate::cookies;
use crate::listener::{Listener, ToListener};
use crate::log;
use crate::middleware::{Middleware, Next};
use crate::router::{Router, Selection};
use crate::{Endpoint, Request, Route};
/// An HTTP server.
///
/// Servers are built up as a combination of *state*, *endpoints* and *middleware*:
///
/// - Server state is user-defined, and is provided via the [`Server::with_state`] function. The
/// state is available as a shared reference to all app endpoints.
///
/// - Endpoints provide the actual application-level code corresponding to
/// particular URLs. The [`Server::at`] method creates a new *route* (using
/// standard router syntax), which can then be used to register endpoints
/// for particular HTTP request types.
///
/// - Middleware extends the base Tide framework with additional request or
/// response processing, such as compression, default headers, or logging. To
/// add middleware to an app, use the [`Server::middleware`] method.
#[allow(missing_debug_implementations)]
pub struct Server<State> {
    router: Arc<Router<State>>,
    state: State,
    middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
}

impl Server<()> {
    /// Create a new Tide server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// let mut app = tide::new();
    /// app.at("/").get(|_| async { Ok("Hello, world!") });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) }) }
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl Default for Server<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State: Clone + Send + Sync + 'static> Server<State> {
    /// Create a new Tide server with shared application scoped state.
    ///
    /// Application scoped state is useful for storing items
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use async_std::task::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// use tide::Request;
    ///
    /// /// The shared application state.
    /// #[derive(Clone)]
    /// struct State {
    ///     name: String,
    /// }
    ///
    /// // Define a new instance of the state.
    /// let state = State {
    ///     name: "Nori".to_string()
    /// };
    ///
    /// // Initialize the application with state.
    /// let mut app = tide::with_state(state);
    /// app.at("/").get(|req: Request<State>| async move {
    ///     Ok(format!("Hello, {}!", &req.state().name))
    /// });
    /// app.listen("127.0.0.1:8080").await?;
    /// #
    /// # Ok(()) }) }
    /// ```
    pub fn with_state(state: State) -> Self {
        let mut server = Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(vec![]),
            state,
        };
        server.with(cookies::CookiesMiddleware::new());
        #[cfg(feature = "logger")]
        server.with(log::LogMiddleware::new());
        server
    }

    /// Add a new route at the given `path`, relative to root.
    ///
    /// Routing means mapping an HTTP request to an endpoint. Here Tide applies
    /// a "table of contents" approach, which makes it easy to see the overall
    /// app structure. Endpoints are selected solely by the path and HTTP method
    /// of a request: the path determines the resource and the HTTP verb the
    /// respective endpoint of the selected resource. Example:
    ///
    /// ```rust,no_run
    /// # let mut app = tide::Server::new();
    /// app.at("/").get(|_| async { Ok("Hello, world!") });
    /// ```
    ///
    /// A path is comprised of zero or many segments, i.e. non-empty strings
    /// separated by '/'. There are two kinds of segments: concrete and
    /// wildcard. A concrete segment is used to exactly match the respective
    /// part of the path of the incoming request. A wildcard segment on the
    /// other hand extracts and parses the respective part of the path of the
    /// incoming request to pass it along to the endpoint as an argument. A
    /// wildcard segment is written as `:name`, which creates an endpoint
    /// parameter called `name`. It is not possible to define wildcard segments
    /// with different names for otherwise identical paths.
    ///
    /// Alternatively a wildcard definitions can start with a `*`, for example
    /// `*path`, which means that the wildcard will match to the end of given
    /// path, no matter how many segments are left, even nothing.
    ///
    /// The name of the parameter can be omitted to define a path that matches
    /// the required structure, but where the parameters are not required.
    /// `:` will match a segment, and `*` will match an entire path.
    ///
    /// Here are some examples omitting the HTTP verb based endpoint selection:
    ///
    /// ```rust,no_run
    /// # let mut app = tide::Server::new();
    /// app.at("/");
    /// app.at("/hello");
    /// app.at("add_two/:num");
    /// app.at("files/:user/*");
    /// app.at("static/*path");
    /// app.at("static/:context/:");
    /// ```
    ///
    /// There is no fallback route matching, i.e. either a resource is a full
    /// match or not, which means that the order of adding resources has no
    /// effect.
    pub fn at<'a>(&'a mut self, path: &str) -> Route<'a, State> {
        let router = Arc::get_mut(&mut self.router)
            .expect("Registering routes is not possible after the Server has started");
        Route::new(router, path.to_owned())
    }

    /// Add middleware to an application.
    ///
    /// Middleware provides customization of the request/response cycle, such as compression,
    /// logging, or header modification. Middleware is invoked when processing a request, and can
    /// either continue processing (possibly modifying the response) or immediately return a
    /// response. See the [`Middleware`] trait for details.
    ///
    /// Middleware can only be added at the "top level" of an application, and is processed in the
    /// order in which it is applied.
    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        log::trace!("Adding middleware {}", middleware.name());
        let m = Arc::get_mut(&mut self.middleware)
            .expect("Registering middleware is not possible after the Server has started");
        m.push(Arc::new(middleware));
        self
    }

    /// Asynchronously serve the app with the supplied listener. For more details, see [Listener] and [ToListener]
    pub async fn listen<TL: ToListener<State>>(self, listener: TL) -> io::Result<()> {
        listener.to_listener()?.listen(self).await
    }

    /// Respond to a `Request` with a `Response`.
    ///
    /// This method is useful for testing endpoints directly,
    /// or for creating servers over custom transports.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[async_std::main]
    /// # async fn main() -> http_types::Result<()> {
    /// #
    /// use tide::http::{Url, Method, Request, Response};
    ///
    /// let mut app = tide::new();
    /// app.at("/").get(|_| async { Ok("hello world") });
    ///
    /// let req = Request::new(Method::Get, Url::parse("https://example.com")?);
    /// let res: Response = app.respond(req).await?;
    ///
    /// assert_eq!(res.status(), 200);
    /// #
    /// # Ok(()) }
    /// ```
    pub async fn respond<R>(&self, req: impl Into<http_types::Request>) -> http_types::Result<R>
    where
        R: From<http_types::Response>,
    {
        let req = req.into();
        let Self {
            router,
            state,
            middleware,
        } = self.clone();

        let method = req.method().to_owned();
        let Selection { endpoint, params } = router.route(&req.url().path(), method);
        let route_params = vec![params];
        let req = Request::new(state, req, route_params);

        let next = Next {
            endpoint,
            next_middleware: &middleware,
        };

        let res = next.run(req).await;
        let res: http_types::Response = res.into();
        Ok(res.into())
    }

    /// Gets a reference to the server's state. This is useful for testing and nesting:
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[derive(Clone)] struct SomeAppState;
    /// let mut app = tide::with_state(SomeAppState);
    /// let mut admin = tide::with_state(app.state().clone());
    /// admin.at("/").get(|_| async { Ok("nested app with cloned state") });
    /// app.at("/").nest(admin);
    /// ```
    pub fn state(&self) -> &State {
        &self.state
    }
}

impl<State: Clone> Clone for Server<State> {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            state: self.state.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

#[async_trait::async_trait]
impl<State: Clone + Sync + Send + 'static, InnerState: Clone + Sync + Send + 'static>
    Endpoint<State> for Server<InnerState>
{
    async fn call(&self, req: Request<State>) -> crate::Result {
        let Request {
            req,
            mut route_params,
            ..
        } = req;
        let path = req.url().path().to_owned();
        let method = req.method().to_owned();
        let router = self.router.clone();
        let middleware = self.middleware.clone();
        let state = self.state.clone();

        let Selection { endpoint, params } = router.route(&path, method);
        route_params.push(params);
        let req = Request::new(state, req, route_params);

        let next = Next {
            endpoint,
            next_middleware: &middleware,
        };

        Ok(next.run(req).await)
    }
}

#[cfg(test)]
mod test {
    use crate as tide;

    #[test]
    fn allow_nested_server_with_same_state() {
        let inner = tide::new();
        let mut outer = tide::new();
        outer.at("/foo").get(inner);
    }

    #[test]
    fn allow_nested_server_with_different_state() {
        let inner = tide::with_state(1);
        let mut outer = tide::new();
        outer.at("/foo").get(inner);
    }
}
