//! An HTTP server

use async_std::future::Future;
use async_std::io;
use async_std::net::ToSocketAddrs;
use async_std::sync::Arc;
use async_std::task::{Context, Poll};
use http_service::HttpService;

use std::fmt::Debug;
use std::pin::Pin;

use crate::cookies;
use crate::log;
use crate::middleware::{Middleware, Next};
use crate::router::{Router, Selection};
use crate::utils::BoxFuture;
use crate::{Endpoint, Request};

mod route;
mod serve_dir;

pub use route::Route;

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

impl Server<()> {
    /// Create a new Tide server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::executor::block_on;
    /// # fn main() -> Result<(), std::io::Error> { block_on(async {
    /// #
    /// let mut app = tide::new();
    /// app.at("/").get(|_| async move { Ok("Hello, world!") });
    /// app.listen("127.0.0.1:8080").await
    /// #
    /// # }) }
    /// ```
    pub fn new() -> Server<()> {
        Self::with_state(())
    }
}

impl Default for Server<()> {
    fn default() -> Server<()> {
        Self::new()
    }
}

impl<State: Send + Sync + 'static> Server<State> {
    /// Create a new Tide server with shared global state.
    ///
    /// Global state is useful for storing items
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::executor::block_on;
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
    /// app.listen("127.0.0.1:8080").await
    /// #
    /// # }) }
    /// ```
    pub fn with_state(state: State) -> Server<State> {
        let mut server = Server {
            router: Arc::new(Router::new()),
            middleware: Arc::new(vec![]),
            state: Arc::new(state),
        };
        server.middleware(cookies::CookiesMiddleware::new());
        server.middleware(log::LogMiddleware::new());
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
    /// app.at("/").get(|_| async move { Ok("Hello, world!") });
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
    pub fn at<'a>(&'a mut self, path: &'a str) -> Route<'a, State> {
        let router = Arc::get_mut(&mut self.router)
            .expect("Registering routes is not possible after the Server has started");
        Route::new(router, path.to_owned())
    }

    /// Add middleware to an application.
    ///
    /// Middleware provides application-global customization of the
    /// request/response cycle, such as compression, logging, or header
    /// modification. Middleware is invoked when processing a request, and can
    /// either continue processing (possibly modifying the response) or
    /// immediately return a response. See the [`Middleware`] trait for details.
    ///
    /// Middleware can only be added at the "top level" of an application,
    /// and is processed in the order in which it is applied.
    pub fn middleware<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State> + Debug,
    {
        log::trace!("Adding middleware {:?}", middleware);
        let m = Arc::get_mut(&mut self.middleware)
            .expect("Registering middleware is not possible after the Server has started");
        m.push(Arc::new(middleware));
        self
    }

    /// Asynchronously serve the app at the given address.
    #[cfg(feature = "h1-server")]
    pub async fn listen(self, addr: impl ToSocketAddrs) -> std::io::Result<()> {
        let listener = async_std::net::TcpListener::bind(addr).await?;

        let addr = format!("http://{}", listener.local_addr()?);
        log::info!("Server is listening on: {}", addr);
        let mut server = http_service_h1::Server::new(addr, listener.incoming(), self);

        server.run().await
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

#[derive(Debug)]
pub struct ReadyFuture;

impl Future for ReadyFuture {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Ok(()))
    }
}

impl<State: Sync + Send + 'static> HttpService for Server<State> {
    type Connection = ();
    type ConnectionFuture = ReadyFuture;
    type ConnectionError = io::Error;
    type ResponseFuture = BoxFuture<'static, Result<http_service::Response, http_types::Error>>;
    type ResponseError = http_types::Error;

    fn connect(&self) -> Self::ConnectionFuture {
        ReadyFuture {}
    }

    fn respond(&self, _conn: (), req: http_service::Request) -> Self::ResponseFuture {
        let req = Request::new(self.state.clone(), req, Vec::new());
        let service = self.clone();
        Box::pin(async move {
            match service.call(req).await {
                Ok(value) => {
                    let res = value.into();
                    // We assume that if an error was manually cast to a
                    // Response that we actually want to send the body to the
                    // client. At this point we don't scrub the message.
                    Ok(res)
                }
                Err(err) => {
                    let mut res = http_types::Response::new(err.status());
                    res.set_content_type(http_types::mime::PLAIN);
                    // Only send the message if it is a non-500 range error. All
                    // errors default to 500 by default, so sending the error
                    // body is opt-in at the call site.
                    if !res.status().is_server_error() {
                        res.set_body(err.to_string());
                    }
                    Ok(res)
                }
            }
        })
    }
}

impl<State: Sync + Send + 'static, InnerState: Sync + Send + 'static> Endpoint<State>
    for Server<InnerState>
{
    fn call<'a>(&'a self, req: Request<State>) -> BoxFuture<'a, crate::Result> {
        let Request {
            request: req,
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

            let res = next.run(req).await?;
            Ok(res)
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
