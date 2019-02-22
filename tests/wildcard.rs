#![feature(futures_api, async_await)]

use futures::{executor::block_on, prelude::*};
use http_service::{Body, HttpService, Request, Response};
use tide::{
    head::{Named, NamedSegment},
    Server,
};

struct TestBackend<T: HttpService> {
    service: T,
    connection: T::Connection,
}

impl<T: HttpService> TestBackend<T> {
    fn wrap(service: T) -> Result<Self, <T::ConnectionFuture as TryFuture>::Error> {
        let connection = block_on(service.connect().into_future())?;
        Ok(Self {
            service,
            connection,
        })
    }

    fn simulate(&mut self, req: Request) -> Result<Response, <T::Fut as TryFuture>::Error> {
        block_on(
            self.service
                .respond(&mut self.connection, req)
                .into_future(),
        )
    }
}

struct Number(i32);

impl NamedSegment for Number {
    const NAME: &'static str = "num";
}

impl std::str::FromStr for Number {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(|num| Number(num))
    }
}

async fn add_one(Named(Number(num)): Named<Number>) -> String {
    (num + 1).to_string()
}

fn make_server() -> TestBackend<Server<()>> {
    let mut app = tide::App::new(());
    app.at("/add_one/{num}").get(add_one);
    TestBackend::wrap(app.into_http_service()).unwrap()
}

#[test]
fn wildcard() {
    let mut server = make_server();

    let req = http::Request::get("/add_one/3")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"4");

    let req = http::Request::get("/add_one/-7")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"-6");
}

#[test]
fn invalid_segment_error() {
    let mut server = make_server();

    let req = http::Request::get("/add_one/a")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 400);
}

#[test]
fn not_found_error() {
    let mut server = make_server();

    let req = http::Request::get("/add_one/").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 404);
}
