use futures::{
    compat::{Compat, Future01CompatExt},
    future::{self, FutureObj},
    prelude::*,
};
use hyper::service::Service;
use slog::{info, Logger};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{
    body::Body,
    config::{Config, Env},
    extract::Extract,
    middleware::{logger::RootLogger, RequestContext},
    router::{Resource, RouteResult, Router},
    typemap::STATE,
    Middleware, Request, Response, RouteMatch,
};

/// The top-level type for setting up a Tide application.
///
/// Apps are equipped with a handle to their own state (`Data`), which is available to all endpoints.
/// This is a "handle" because it must be `Clone`, and endpoints are invoked with a fresh clone.
/// They also hold a top-level router.
pub struct App<Data> {
    data: Data,
    router: Router<Data>,
}

impl<Data: Clone + Send + Sync + 'static> App<Data> {
    /// Set up a new app with some initial `data`.
    pub fn new(data: Data) -> App<Data> {
        let logger = RootLogger::init();
        let config = Config { env: Env::Dev };

        let mut map = STATE.write().unwrap();
        // store config in app state
        map.insert(config);
        let mut app = App {
            data,
            router: Router::new(),
        };

        // Add RootLogger as a default middleware
        app.middleware(logger);

        app
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

    /// Apply `middleware` to the whole app. Note that the order of nesting subrouters and applying
    /// middleware matters; see `Router` for details.
    pub fn middleware(&mut self, middleware: impl Middleware<Data> + 'static) -> &mut Self {
        self.router.middleware(middleware);
        self
    }

    fn into_server(self) -> Server<Data> {
        Server {
            data: self.data,
            router: Arc::new(self.router),
        }
    }

    /// Start serving the app at the given address.
    ///
    /// Blocks the calling thread indefinitely.
    pub fn serve<A: std::net::ToSocketAddrs>(self, addr: A) {
        let server: Server<Data> = self.into_server();

        // TODO: be more robust
        let addr = addr.to_socket_addrs().unwrap().next().unwrap();

        let server = hyper::Server::bind(&addr)
            .serve(move || {
                let res: Result<_, std::io::Error> = Ok(server.clone());
                res
            })
            .compat()
            .map(|_| {
                let res: Result<(), ()> = Ok(());
                res
            })
            .compat();

        let map = STATE.read().unwrap();
        let logger = map.get::<Logger>().unwrap();
        let config = map.get::<Config>().unwrap();

        info!(
            logger,
            "Environment: {:?} Starting server on: {}", config.env, addr
        );
        hyper::rt::run(server);
    }
}

#[derive(Clone)]
struct Server<Data> {
    data: Data,
    router: Arc<Router<Data>>,
}

impl<Data: Clone + Send + Sync + 'static> Service for Server<Data> {
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = std::io::Error;
    type Future = Compat<FutureObj<'static, Result<http::Response<hyper::Body>, Self::Error>>>;

    fn call(&mut self, req: http::Request<hyper::Body>) -> Self::Future {
        let data = self.data.clone();
        let router = self.router.clone();

        let req = req.map(Body::from);
        let path = req.uri().path().to_owned();
        let method = req.method().to_owned();

        FutureObj::new(Box::new(
            async move {
                if let Some(RouteResult {
                    endpoint,
                    params,
                    middleware,
                }) = router.route(&path, &method)
                {
                    let ctx = RequestContext {
                        app_data: data,
                        req,
                        params,
                        endpoint,
                        next_middleware: middleware,
                    };
                    let res = await!(ctx.next());
                    Ok(res.map(Into::into))
                } else {
                    Ok(http::Response::builder()
                        .status(http::status::StatusCode::NOT_FOUND)
                        .body(hyper::Body::empty())
                        .unwrap())
                }
            },
        ))
        .compat()
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
    fn extract(data: &mut T, req: &mut Request, params: &RouteMatch<'_>) -> Self::Fut {
        future::ok(AppData(data.clone()))
    }
}
