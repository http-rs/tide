use futures::future::{self, FutureObj};
use http_service::HttpService;
use std::{
    any::Any,
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{
    configuration::{Configuration, Store},
    endpoint::BoxedEndpoint,
    endpoint::Endpoint,
    extract::Extract,
    middleware::{logger::RootLogger, RequestContext},
    router::{EndpointData, Resource, RouteResult, Router},
    Middleware, Request, Response, RouteMatch,
};

/// The top-level type for setting up a Tide application.
///
/// Apps are equipped with a handle to their own state (`Data`), which is available to all endpoints.
/// This is a "handle" because it must be `Clone`, and endpoints are invoked with a fresh clone.
/// They also hold a top-level router.
///
/// # Examples
///
/// You can start a simple Tide application that listens for `GET` requests at path `/hello`
/// on `127.0.0.1:8181` with:
///
/// ```rust, no_run
/// #![feature(async_await)]
///
/// let mut app = tide::App::new(());
/// app.at("/hello").get(async || "Hello, world!");
/// app.serve()
/// ```
///
/// `App` state can be modeled with an underlying `Data` handle for a cloneable type `T`. Endpoints
/// can receive a fresh clone of that handle (in addition to data extracted from the request) by
/// defining a parameter of type `AppData<T>`:
///
/// ```rust, no_run
/// #![feature(async_await, futures_api)]
///
/// use std::sync::Arc;
/// use std::sync::Mutex;
/// use tide::AppData;
/// use tide::body;
///
/// #[derive(Clone, Default)]
/// struct Database {
///     contents: Arc<Mutex<Vec<String>>>,
/// }
///
/// async fn insert(
///     mut db: AppData<Database>,
///     msg: body::Str,
/// ) -> String {
///     // insert into db
///     # String::from("")
/// }
///
/// fn main() {
///     let mut app = tide::App::new(Database::default());
///     app.at("/messages/insert").post(insert);
///     app.serve()
/// }
/// ```
///
/// Where to go from here: Please see [`Router`](struct.Router.html) and [`Endpoint`](trait.Endpoint.html)
/// for further examples.
///
pub struct App<Data> {
    data: Data,
    router: Router<Data>,
    default_handler: EndpointData<Data>,
}

impl<Data: Clone + Send + Sync + 'static> App<Data> {
    /// Set up a new app with some initial `data`.
    pub fn new(data: Data) -> App<Data> {
        let logger = RootLogger::new();
        let mut app = App {
            data,
            router: Router::new(),
            default_handler: EndpointData {
                endpoint: BoxedEndpoint::new(async || http::status::StatusCode::NOT_FOUND),
                store: Store::new(),
            },
        };

        // Add RootLogger as a default middleware
        app.middleware(logger);
        app.setup_configuration();

        app
    }

    // Add default configuration
    fn setup_configuration(&mut self) {
        let config = Configuration::build().finalize();
        self.config(config);
    }

    /// Get the top-level router.
    pub fn router(&mut self) -> &mut Router<Data> {
        &mut self.router
    }

    /// Add a new resource at `path`.
    /// See [Router.at](struct.Router.html#method.at) for details.
    pub fn at<'a>(&'a mut self, path: &'a str) -> Resource<'a, Data> {
        self.router.at(path)
    }

    /// Set the default handler for the app, a fallback function when there is no match to the route requested
    pub fn default_handler<T: Endpoint<Data, U>, U>(
        &mut self,
        handler: T,
    ) -> &mut EndpointData<Data> {
        let endpoint = EndpointData {
            endpoint: BoxedEndpoint::new(handler),
            store: self.router.store_base.clone(),
        };
        self.default_handler = endpoint;
        &mut self.default_handler
    }

    /// Apply `middleware` to the whole app. Note that the order of nesting subrouters and applying
    /// middleware matters; see `Router` for details.
    pub fn middleware(&mut self, middleware: impl Middleware<Data> + 'static) -> &mut Self {
        self.router.middleware(middleware);
        self
    }

    /// Add a default configuration `item` for the whole app.
    pub fn config<T: Any + Debug + Clone + Send + Sync>(&mut self, item: T) -> &mut Self {
        self.router.config(item);
        self
    }

    pub fn get_item<T: Any + Debug + Clone + Send + Sync>(&self) -> Option<&T> {
        self.router.get_item()
    }

    fn into_server(mut self) -> Server<Data> {
        self.router.apply_default_config();
        Server {
            data: self.data,
            router: Arc::new(self.router),
            default_handler: Arc::new(self.default_handler),
        }
    }

    /// Start serving the app at the given address.
    ///
    /// Blocks the calling thread indefinitely.
    pub fn serve(self) {
        let configuration = self.get_item::<Configuration>().unwrap();
        let addr = format!("{}:{}", configuration.address, configuration.port)
            .parse::<std::net::SocketAddr>()
            .unwrap();

        println!("Server is listening on: http://{}", addr);

        crate::serve::serve(self.into_server(), addr);
    }
}

#[derive(Clone)]
struct Server<Data> {
    data: Data,
    router: Arc<Router<Data>>,
    default_handler: Arc<EndpointData<Data>>,
}

impl<Data> HttpService for Server<Data>
    where Data: Clone + Send + Sync + 'static
{
    type Connection = ();
    type ConnectionFuture = future::Ready<Result<(), std::io::Error>>;
    type Fut = FutureObj<'static, Result<http_service::Response, std::io::Error>>;

    fn connect(&self) -> Self::ConnectionFuture {
        future::ok(())
    }

    fn respond(&self, _conn: &mut (), req: http_service::Request) -> Self::Fut {
        let data = self.data.clone();
        let router = self.router.clone();
        let default_handler = self.default_handler.clone();
        let path = req.uri().path().to_owned();
        let method = req.method().to_owned();

        FutureObj::new(Box::new(
            async move {
                let RouteResult {
                    endpoint,
                    params,
                    middleware,
                } = router.route(&path, &method, &default_handler);

                let ctx = RequestContext {
                    app_data: data,
                    req,
                    params,
                    endpoint,
                    next_middleware: middleware,
                };
                Ok(await!(ctx.next()))
            }
        ))
    }
}

/// An extractor for accessing app data.
///
/// Endpoints can use `AppData<T>` to gain a handle to the data (of type `T`) originally injected into their app.
pub struct AppData<T>(pub T);

impl<T> Deref for AppData<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T> DerefMut for AppData<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Clone + Send + 'static> Extract<T> for AppData<T> {
    type Fut = future::Ready<Result<Self, Response>>;
    fn extract(
        data: &mut T,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        future::ok(AppData(data.clone()))
    }
}
