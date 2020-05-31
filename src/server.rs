//! An HTTP server

use async_std::io;
use async_std::net::ToSocketAddrs;
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::task;

use crate::cookies;
use crate::log;
use crate::middleware::{Middleware, Next};
use crate::router::{Router, Selection};
use crate::utils::BoxFuture;
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
/////
///// # Hello, world!
/////
///// You can start a simple Tide application that listens for `GET` requests at path `/hello`
///// on `127.0.0.1:8000` with:
/////
///// ```rust, no_run
/////
///// let mut app = tide::Server::new();
///// app.at("/hello").get(|_| async move {"Hello, world!"});
///// // app.run("127.0.0.1:8000").unwrap();
///// ```
/////
///// # Routing and parameters
/////
///// Tide's routing system is simple and similar to many other frameworks. It
///// uses `:foo` for "wildcard" URL segments, and `*foo` to match the rest of a
///// URL (which may include multiple segments). Here's an example using wildcard
///// segments as parameters to endpoints:
/////
///// ```no_run
///// use tide::error::ResultExt;
/////
///// async fn hello(cx: tide::Request<()>) -> tide::Result<String> {
/////     let user: String = cx.param("user")?;
/////     Ok(format!("Hello, {}!", user))
///// }
/////
///// async fn goodbye(cx: tide::Request<()>) -> tide::Result<String> {
/////     let user: String = cx.param("user")?;
/////     Ok(format!("Goodbye, {}.", user))
///// }
/////
///// let mut app = tide::Server::new();
/////
///// app.at("/hello/:user").get(hello);
///// app.at("/goodbye/:user").get(goodbye);
///// app.at("/").get(|_| async move {
/////     "Use /hello/{your name} or /goodbye/{your name}"
///// });
/////
///// // app.run("127.0.0.1:8000").unwrap();
///// ```
/////
///// You can learn more about routing in the [`Server::at`] documentation.
/////
///// # Serverlication state
/////
///// ```rust,no_run
///// use http_types::status::StatusCode;
///// use serde::{Deserialize, Serialize};
///// use std::sync::Mutex;
///// use tide::{error::ResultExt, Server, Request, Result};
/////
///// #[derive(Default)]
///// struct Database {
/////     contents: Mutex<Vec<Message>>,
///// }
/////
///// #[derive(Serialize, Deserialize, Clone)]
///// struct Message {
/////     author: Option<String>,
/////     contents: String,
///// }
/////
///// impl Database {
/////     fn insert(&self, msg: Message) -> usize {
/////         let mut table = self.contents.lock().unwrap();
/////         table.push(msg);
/////         table.len() - 1
/////     }
/////
/////     fn get(&self, id: usize) -> Option<Message> {
/////         self.contents.lock().unwrap().get(id).cloned()
/////     }
///// }
/////
///// async fn new_message(mut cx: Request<Database>) -> Result<String> {
/////     let msg = cx.body_json().await?;
/////     Ok(cx.state().insert(msg).to_string())
///// }
/////
///// async fn get_message(cx: Request<Database>) -> Result {
/////     let id = cx.param("id").unwrap();
/////     if let Some(msg) = cx.state().get(id) {
/////         Ok(response::json(msg))
/////     } else {
/////         Err(StatusCode::NOT_FOUND)?
/////     }
///// }
/////
///// fn main() {
/////     let mut app = Server::with_state(Database::default());
/////     app.at("/message").post(new_message);
/////     app.at("/message/:id").get(get_message);
/////     // app.run("127.0.0.1:8000").unwrap();
///// }
///// ```
#[allow(missing_debug_implementations)]
pub struct Server<State> {
    router: Arc<Router<State>>,
    state: Arc<State>,
    middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
}

fn is_transient_error(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::ConnectionRefused
        || e.kind() == io::ErrorKind::ConnectionAborted
        || e.kind() == io::ErrorKind::ConnectionReset
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

impl<State: Send + Sync + 'static> Server<State> {
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
        Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(vec![]),
            state: Arc::new(state),
        }
    }

    /// Add the logging middleware to an application.
    ///
    /// ```rust,no_run
    /// # let app = tide::Server::new().with_logging();
    /// ```
    pub fn with_logging(mut self) -> Self {
        self.middleware(log::LogMiddleware::new());
        self
    }

    /// Add the cookies middleware to an application.
    ///
    /// ```rust,no_run
    /// # let app = tide::Server::new().with_cookies();
    /// ```
    pub fn with_cookies(mut self) -> Self {
        self.middleware(cookies::CookiesMiddleware::new());
        self
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
    pub fn middleware<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        log::trace!("Adding middleware {}", middleware.name());
        let m = Arc::get_mut(&mut self.middleware)
            .expect("Registering middleware is not possible after the Server has started");
        m.push(Arc::new(middleware));
        self
    }

    #[cfg(feature = "h1-server")]
    fn handle_tcp(self, stream: async_std::net::TcpStream) {
        let local_addr = stream.local_addr().ok();
        let peer_addr = stream.peer_addr().ok();
        task::spawn(async move {
            let result = async_h1::accept(stream, |mut req| async {
                req.set_local_addr(local_addr);
                req.set_peer_addr(peer_addr);
                self.respond(req).await
            })
            .await;

            if let Err(error) = result {
                log::error!("async-h1 error", { error: error.to_string() });
            }
        });
    }

    /// Asynchronously serve the app at the given address.
    #[cfg(feature = "h1-server")]
    pub async fn listen(self, addr: impl ToSocketAddrs) -> io::Result<()> {
        let listener = async_std::net::TcpListener::bind(addr).await?;

        let addr = format!("http://{}", listener.local_addr()?);
        let tls = false;
        let target = if cfg!(debug_assertions) {
            "dev"
        } else {
            "release"
        };
        log::info!("Server listening on {}", addr, { address: addr, target: target, tls: tls });

        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            match stream {
                Err(ref e) if is_transient_error(e) => continue,
                Err(error) => {
                    let delay = std::time::Duration::from_millis(500);
                    crate::log::error!("Error: {}. Pausing for {:?}.", error, delay);
                    task::sleep(delay).await;
                    continue;
                }
                Ok(stream) => self.clone().handle_tcp(stream),
            };
        }

        Ok(())
    }

    /// Asynchronously serve the app at the given address.
    #[cfg(all(feature = "h1-server", unix))]
    pub async fn listen_unix(self, addr: impl AsRef<async_std::path::Path>) -> io::Result<()> {
        let listener = async_std::os::unix::net::UnixListener::bind(addr).await?;
        let tls = false;
        let target = if cfg!(debug_assertions) {
            "dev"
        } else {
            "release"
        };

        let addr = format!("unix://{:?}", listener.local_addr()?);
        log::info!("Server listening on {}", addr, { address: addr, target: target, tls: tls });

        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            match stream {
                Err(ref e) if is_transient_error(e) => continue,
                Err(error) => {
                    let delay = std::time::Duration::from_millis(500);
                    crate::log::error!("Error: {}. Pausing for {:?}.", error, delay);
                    task::sleep(delay).await;
                    continue;
                }
                Ok(stream) => self.clone().handle_unix(stream),
            };
        }
        Ok(())
    }

    #[cfg(all(feature = "h1-server", unix))]
    fn handle_unix(self, stream: async_std::os::unix::net::UnixStream) {
        task::spawn(async move {
            let local_addr = stream.local_addr().ok().map(|addr| format!("{:?}", addr));
            let peer_addr = stream.peer_addr().ok().map(|addr| format!("{:?}", addr));

            let result = async_h1::accept(stream, |mut req| async {
                req.set_local_addr(local_addr.as_ref());
                req.set_peer_addr(peer_addr.as_ref());
                self.respond(req).await
            })
            .await;

            if let Err(error) = result {
                log::error!("async-h1 error", { error: error.to_string() });
            }
        });
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
}

impl<State> Clone for Server<State> {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            state: self.state.clone(),
            middleware: self.middleware.clone(),
        }
    }
}

impl<State: Sync + Send + 'static, InnerState: Sync + Send + 'static> Endpoint<State>
    for Server<InnerState>
{
    fn call<'a>(&'a self, req: Request<State>) -> BoxFuture<'a, crate::Result> {
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

        Box::pin(async move {
            let Selection { endpoint, params } = router.route(&path, method);
            route_params.push(params);
            let req = Request::new(state, req, route_params);

            let next = Next {
                endpoint,
                next_middleware: &middleware,
            };

            Ok(next.run(req).await)
        })
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
