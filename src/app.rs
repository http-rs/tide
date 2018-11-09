use futures::{
    compat::{Compat, Future01CompatExt},
    future::{self, FutureObj},
    task::Spawn,
    prelude::*,
};
use hyper::service::Service;
use std::{sync::Arc, ops::{Deref, DerefMut}, marker::Unpin};

use crate::{
    router::{Resource, Router},
    Request,
    extract::Extract,
    RouteMatch, 
    Response, 
    body::Body,
    Middleware,
};

/// The top-level type for setting up a Tide application.
/// 
/// Apps are equipped with a handle to their own state (`Data`), which is available to all endpoints.
/// This is a "handle" because it must be `Clone`, and endpoints are invoked with a fresh clone.
/// They also hold a top-level router.
pub struct App<Data> {
    data: Data,
    router: Router<Data>,
    middleware: Vec<Box<dyn Middleware<Data> + Send + Sync>>,
}

impl<Data: Clone + Send + Sync + 'static> App<Data> {
    /// Set up a new app with some initial `data`.
    pub fn new(data: Data) -> App<Data> {
        App {
            data,
            router: Router::new(),
            middleware: Vec::new(),
        }
    }

    /// Add a new resource at `path`.
    pub fn at<'a>(&'a mut self, path: &'a str) -> &mut Resource<Data> {
        self.router.at(path)
    }

    /// Apply `middleware` to the whole app.
    pub fn middleware(&mut self, middleware: impl Middleware<Data> + 'static) -> &mut Self {
        self.middleware.push(Box::new(middleware));
        self
    }

    fn into_service(self) -> Server<Data> {
        Server {
            data: self.data,
            router: Arc::new(self.router),
            middleware: Arc::new(self.middleware),
        }
    }

    /// Start serving the app at the given address.
    ///
    /// Blocks the calling thread indefinitely.
    #[cfg(feature = "tokio-runtime")]
    pub fn serve<A: std::net::ToSocketAddrs>(self, addr: A) {
        let service = self.into_service();

        // TODO: be more robust
        let addr = addr.to_socket_addrs().unwrap().next().unwrap();

        let server = hyper::Server::bind(&addr).serve(move || {
            let res: Result<_, std::io::Error> = Ok(service.clone());            
            res
        }).compat().map(|_| {
            let res: Result<(), ()> = Ok(());
            res
        }).compat();
        hyper::rt::run(server);
    }

    /// Start serving the app on a stream of incoming connections.
    ///
    /// This function provides support for running the server on
    /// non-Tokio executors. If using the Tokio runtime, `serve`
    /// should be used instead.
    pub fn serve_incoming<St, Sp>(
        self,
        incoming_stream: St,
        spawn: Sp,
    ) -> impl Future<Output = Result<(), impl std::error::Error + Send + Sync + 'static>>
    where
        St: TryStream + Unpin + 'static,
        St::Ok: AsyncRead + AsyncWrite + Send + 'static,
        St::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        Sp: Send + Clone + 'static,
        for<'a> &'a Sp: Spawn,
    {
        let service = self.into_service();
        hyper::Server::builder(
            incoming_stream
                .map_ok(|con| con.compat())
                .compat()
        )
            .executor(Compat::new(spawn))
            .serve(move || Ok::<_, std::io::Error>(service.clone()))
            .compat()
    }
}

#[derive(Clone)]
struct Server<Data> {
    data: Data,
    router: Arc<Router<Data>>,
    middleware: Arc<Vec<Box<dyn Middleware<Data> + Send + Sync>>>,
}

impl<Data: Clone + Send + Sync + 'static> Service for Server<Data> {
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = std::io::Error;
    type Future = Compat<FutureObj<'static, Result<http::Response<hyper::Body>, Self::Error>>>;

    fn call(&mut self, req: http::Request<hyper::Body>) -> Self::Future {
        let mut data = self.data.clone();
        let router = self.router.clone();
        let middleware = self.middleware.clone();

        let mut req = req.map(Body::from);            
        let path = req.uri().path().to_owned();
        let method = req.method().to_owned();

        FutureObj::new(Box::new(async move {            
            if let Some((endpoint, params)) = router.route(&path, &method) {                
                for m in middleware.iter() {
                    match await!(m.request(&mut data, req, &params)) {
                        Ok(new_req) => req = new_req,
                        Err(resp) => return Ok(resp.map(Into::into)),
                    }                    
                }

                let (head, mut resp) = await!(endpoint.call(data.clone(), req, params));

                for m in middleware.iter() {
                    resp = await!(m.response(&mut data, &head, resp))
                }

                Ok(resp.map(Into::into))
            } else {
                Ok(http::Response::builder().status(http::status::StatusCode::NOT_FOUND).body(hyper::Body::empty()).unwrap())
            }
        })).compat()
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
        params: &RouteMatch<'_>,
    ) -> Self::Fut {
        future::ok(AppData(data.clone()))
    }
}
