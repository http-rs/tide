//! `HttpService` server that uses Hyper as backend.
#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![feature(futures_api, async_await, await_macro)]

use futures::{
    compat::{Compat, Compat01As03, Future01CompatExt},
    future::FutureObj,
    prelude::*,
};
use http_service::{Body, HttpService};
use std::{net::SocketAddr, sync::Arc};

// Wrapper type to allow us to provide a blanket `MakeService` impl
struct WrapHttpService<H> {
    service: Arc<H>,
}

// Wrapper type to allow us to provide a blanket `Service` impl
struct WrapConnection<H: HttpService> {
    service: Arc<H>,
    connection: H::Connection,
}

impl<H, Ctx> hyper::service::MakeService<Ctx> for WrapHttpService<H>
where
    H: HttpService,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = std::io::Error;
    type Service = WrapConnection<H>;
    type Future = Compat<FutureObj<'static, Result<Self::Service, Self::Error>>>;
    type MakeError = std::io::Error;

    fn make_service(&mut self, _ctx: Ctx) -> Self::Future {
        let service = self.service.clone();
        let error = std::io::Error::from(std::io::ErrorKind::Other);
        FutureObj::new(Box::new(
            async move {
                let connection = await!(service.connect().into_future()).map_err(|_| error)?;
                Ok(WrapConnection {
                    service,
                    connection,
                })
            },
        ))
        .compat()
    }
}

impl<H> hyper::service::Service for WrapConnection<H>
where
    H: HttpService,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = std::io::Error;
    type Future = Compat<FutureObj<'static, Result<http::Response<hyper::Body>, Self::Error>>>;

    fn call(&mut self, req: http::Request<hyper::Body>) -> Self::Future {
        let error = std::io::Error::from(std::io::ErrorKind::Other);
        let req = req.map(|hyper_body| {
            let stream = Compat01As03::new(hyper_body).map(|c| match c {
                Ok(chunk) => Ok(chunk.into_bytes()),
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            });
            Body::from_stream(stream)
        });
        let fut = self.service.respond(&mut self.connection, req);

        FutureObj::new(Box::new(
            async move {
                let res: http::Response<_> = await!(fut.into_future()).map_err(|_| error)?;
                Ok(res.map(|body| hyper::Body::wrap_stream(body.compat())))
            },
        ))
        .compat()
    }
}

/// Serve the given `HttpService` at the given address, using `hyper` as backend.
pub fn serve<S: HttpService>(s: S, addr: SocketAddr) {
    let service = WrapHttpService {
        service: Arc::new(s),
    };
    let server = hyper::Server::bind(&addr)
        .serve(service)
        .compat()
        .map(|_| {
            let res: Result<(), ()> = Ok(());
            res
        })
        .compat();
    hyper::rt::run(server);
}
