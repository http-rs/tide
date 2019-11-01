use futures::future::{self, BoxFuture};
use http_service::HttpService;
use std::sync::Arc;

use crate::{
    middleware::{Middleware, Next},
    router::{Router, Selection},
    Context, Route,
};

/// The entry point for building a Tide application.
///
/// Apps are built up as a combination of *state*, *endpoints* and *middleware*:
///
/// - Application state is user-defined, and is provided via the [`App::new`]
/// function. The state is available as a shared reference to all app endpoints.
///
/// - Endpoints provide the actual application-level code corresponding to
/// particular URLs. The [`App::at`] method creates a new *route* (using
/// standard router syntax), which can then be used to register endpoints
/// for particular HTTP request types.
///
/// - Middleware extends the base Tide framework with additional request or
/// response processing, such as compression, default headers, or logging. To
/// add middleware to an app, use the [`App::middleware`] method.
///
/// # Hello, world!
///
/// You can start a simple Tide application that listens for `GET` requests at path `/hello`
/// on `127.0.0.1:8000` with:
///
/// ```rust, no_run
/// #![feature(async_await)]
///
/// let mut app = tide::App::new();
/// app.at("/hello").get(async move |_| "Hello, world!");
/// app.serve("127.0.0.1:8000");
/// ```
///
/// # Routing and parameters
///
/// Tide's routing system is simple and similar to many other frameworks. It
/// uses `:foo` for "wildcard" URL segments, and `:foo*` to match the rest of a
/// URL (which may include multiple segments). Here's an example using wildcard
/// segments as parameters to endpoints:
///
/// ```rust, no_run
/// #![feature(async_await)]
///
/// use tide::error::ResultExt;
///
/// async fn hello(cx: tide::Context<()>) -> tide::EndpointResult<String> {
///     let user: String = cx.param("user").client_err()?;
///     Ok(format!("Hello, {}!", user))
/// }
///
/// async fn goodbye(cx: tide::Context<()>) -> tide::EndpointResult<String> {
///     let user: String = cx.param("user").client_err()?;
///     Ok(format!("Goodbye, {}.", user))
/// }
///
/// let mut app = tide::App::new();
///
/// app.at("/hello/:user").get(hello);
/// app.at("/goodbye/:user").get(goodbye);
/// app.at("/").get(async move |_| {
///     "Use /hello/{your name} or /goodbye/{your name}"
/// });
///
/// app.serve("127.0.0.1:8000");
/// ```
///
/// You can learn more about routing in the [`App::at`] documentation.
///
/// # Application state
///
/// ```rust, no_run
/// #![feature(async_await)]
///
/// use http::status::StatusCode;
/// use serde::{Deserialize, Serialize};
/// use std::sync::Mutex;
/// use tide::{error::ResultExt, response, App, Context, EndpointResult};
///
/// #[derive(Default)]
/// struct Database {
///     contents: Mutex<Vec<Message>>,
/// }
///
/// #[derive(Serialize, Deserialize, Clone)]
/// struct Message {
///     author: Option<String>,
///     contents: String,
/// }
///
/// impl Database {
///     fn insert(&self, msg: Message) -> usize {
///         let mut table = self.contents.lock().unwrap();
///         table.push(msg);
///         table.len() - 1
///     }
///
///     fn get(&self, id: usize) -> Option<Message> {
///         self.contents.lock().unwrap().get(id).cloned()
///     }
/// }
///
/// async fn new_message(mut cx: Context<Database>) -> EndpointResult<String> {
///     let msg = cx.body_json().await.client_err()?;
///     Ok(cx.state().insert(msg).to_string())
/// }
///
/// async fn get_message(cx: Context<Database>) -> EndpointResult {
///     let id = cx.param("id").client_err()?;
///     if let Some(msg) = cx.state().get(id) {
///         Ok(response::json(msg))
///     } else {
///         Err(StatusCode::NOT_FOUND)?
///     }
/// }
///
/// fn main() {
///     let mut app = App::with_state(Database::default());
///     app.at("/message").post(new_message);
///     app.at("/message/:id").get(get_message);
///     app.serve("127.0.0.1:8000").unwrap();
/// }
/// ```

#[allow(missing_debug_implementations)]
pub struct App<State> {
    router: Router<State>,
    middleware: Vec<Arc<dyn Middleware<State>>>,
    data: State,
}

impl App<()> {
    /// Create an empty `App`, with no initial middleware or configuration.
    pub fn new() -> App<()> {
        Self::with_state(())
    }
}

impl Default for App<()> {
    fn default() -> App<()> {
        Self::new()
    }
}

impl<State: Send + Sync + 'static> App<State> {
    /// Create an `App`, with initial middleware or configuration.
    pub fn with_state(state: State) -> App<State> {
        App {
            router: Router::new(),
            middleware: Vec::new(),
            data: state,
        }
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
    /// # #![feature(async_await)]
    /// # let mut app = tide::App::new();
    /// app.at("/").get(async move |_| "Hello, world!");
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
    /// Wildcard definitions can be followed by an optional *wildcard
    /// modifier*. Currently, there is only one modifier: `*`, which means that
    /// the wildcard will match to the end of given path, no matter how many
    /// segments are left, even nothing. It is an error to define two wildcard
    /// segments with different wildcard modifiers, or to write other path
    /// segment after a segment with wildcard modifier.
    ///
    /// Here are some examples omitting the HTTP verb based endpoint selection:
    ///
    /// ```rust,no_run
    /// # let mut app = tide::App::new();
    /// app.at("/");
    /// app.at("/hello");
    /// app.at("add_two/:num");
    /// app.at("static/:path*");
    /// ```
    ///
    /// There is no fallback route matching, i.e. either a resource is a full
    /// match or not, which means that the order of adding resources has no
    /// effect.
    pub fn at<'a>(&'a mut self, path: &'a str) -> Route<'a, State> {
        Route::new(&mut self.router, path.to_owned())
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
    pub fn middleware(&mut self, m: impl Middleware<State>) -> &mut Self {
        self.middleware.push(Arc::new(m));
        self
    }

    /// Make this app into an `HttpService`.
    ///
    /// This lower-level method lets you host a Tide application within an HTTP
    /// server of your choice, via the `http_service` interface crate.
    pub fn into_http_service(self) -> Server<State> {
        Server {
            router: Arc::new(self.router),
            data: Arc::new(self.data),
            middleware: Arc::new(self.middleware),
        }
    }

    /// Start serving the app at the given address.
    ///
    /// Blocks the calling thread indefinitely.
    #[cfg(feature = "hyper")]
    pub fn serve(self, addr: impl std::net::ToSocketAddrs) -> std::io::Result<()> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(std::io::ErrorKind::InvalidInput)?;

        println!("Server is listening on: http://{}", addr);
        http_service_hyper::run(self.into_http_service(), addr);
        Ok(())
    }
}

/// An instantiated Tide server.
///
/// This type is useful only in conjunction with the [`HttpService`] trait,
/// i.e. for hosting a Tide app within some custom HTTP server.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Server<State> {
    router: Arc<Router<State>>,
    data: Arc<State>,
    middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
}

impl<State: Sync + Send + 'static> HttpService for Server<State> {
    type Connection = ();
    type ConnectionFuture = future::Ready<Result<(), std::io::Error>>;
    type ResponseFuture = BoxFuture<'static, Result<http_service::Response, std::io::Error>>;

    fn connect(&self) -> Self::ConnectionFuture {
        future::ok(())
    }

    fn respond(&self, _conn: &mut (), req: http_service::Request) -> Self::ResponseFuture {
        let path = req.uri().path().to_owned();
        let method = req.method().to_owned();
        let router = self.router.clone();
        let middleware = self.middleware.clone();
        let data = self.data.clone();

        box_async! {
            let fut = {
                let Selection { endpoint, params } = router.route(&path, method);
                let cx = Context::new(data, req, params);

                let next = Next {
                    endpoint,
                    next_middleware: &middleware,
                };

                next.run(cx)
            };

            Ok(fut.await)
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use std::sync::Arc;

    use super::*;
    use crate::{middleware::Next, router::Selection, Context, Response};

    fn simulate_request<'a, Data: Default + Clone + Send + Sync + 'static>(
        app: &'a App<Data>,
        path: &'a str,
        method: http::Method,
    ) -> BoxFuture<'a, Response> {
        let Selection { endpoint, params } = app.router.route(path, method.clone());

        let data = Arc::new(Data::default());
        let req = http::Request::builder()
            .method(method)
            .body(http_service::Body::empty())
            .unwrap();
        let cx = Context::new(data, req, params);
        let next = Next {
            endpoint,
            next_middleware: &app.middleware,
        };

        next.run(cx)
    }

    #[test]
    fn simple_static() {
        let mut router = App::new();
        router.at("/").get(async move |_| "/");
        router.at("/foo").get(async move |_| "/foo");
        router.at("/foo/bar").get(async move |_| "/foo/bar");

        for path in &["/", "/foo", "/foo/bar"] {
            let res = block_on(simulate_request(&router, path, http::Method::GET));
            let body = block_on(res.into_body().into_vec()).expect("Reading body should succeed");
            assert_eq!(&*body, path.as_bytes());
        }
    }

    #[test]
    fn nested_static() {
        let mut router = App::new();
        router.at("/a").get(async move |_| "/a");
        router.at("/b").nest(|router| {
            router.at("/").get(async move |_| "/b");
            router.at("/a").get(async move |_| "/b/a");
            router.at("/b").get(async move |_| "/b/b");
            router.at("/c").nest(|router| {
                router.at("/a").get(async move |_| "/b/c/a");
                router.at("/b").get(async move |_| "/b/c/b");
            });
            router.at("/d").get(async move |_| "/b/d");
        });
        router.at("/a/a").nest(|router| {
            router.at("/a").get(async move |_| "/a/a/a");
            router.at("/b").get(async move |_| "/a/a/b");
        });
        router.at("/a/b").nest(|router| {
            router.at("/").get(async move |_| "/a/b");
        });

        for failing_path in &["/", "/a/a", "/a/b/a"] {
            let res = block_on(simulate_request(&router, failing_path, http::Method::GET));
            if !res.status().is_client_error() {
                panic!(
                    "Should have returned a client error when router cannot match with path {}",
                    failing_path
                );
            }
        }

        for path in &[
            "/a", "/a/a/a", "/a/a/b", "/a/b", "/b", "/b/a", "/b/b", "/b/c/a", "/b/c/b", "/b/d",
        ] {
            let res = block_on(simulate_request(&router, path, http::Method::GET));
            let body = block_on(res.into_body().into_vec()).expect("Reading body should succeed");
            assert_eq!(&*body, path.as_bytes());
        }
    }

    #[test]
    fn multiple_methods() {
        let mut router = App::new();
        router.at("/a").nest(|router| {
            router.at("/b").get(async move |_| "/a/b GET");
        });
        router.at("/a/b").post(async move |_| "/a/b POST");

        for (path, method) in &[("/a/b", http::Method::GET), ("/a/b", http::Method::POST)] {
            let res = block_on(simulate_request(&router, path, method.clone()));
            assert!(res.status().is_success());
            let body = block_on(res.into_body().into_vec()).expect("Reading body should succeed");
            assert_eq!(&*body, format!("{} {}", path, method).as_bytes());
        }
    }
}
