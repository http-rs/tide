#![feature(async_await)]

use futures::executor::block_on;
use http_service::Body;
use http_service_mock::make_server;
use tide::{error::ResultExt, Context};

async fn add_one(cx: Context<()>) -> Result<String, tide::Error> {
    let num: i64 = cx.param("num").client_err()?;
    Ok((num + 1).to_string())
}

#[test]
fn wildcard() {
    let mut app = tide::App::new();
    app.at("/add_one/:num").get(add_one);
    let mut server = make_server(app.into_http_service()).unwrap();

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
    let mut app = tide::App::new();
    app.at("/add_one/:num").get(add_one);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/add_one/a")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 400);
}

#[test]
fn not_found_error() {
    let mut app = tide::App::new();
    app.at("/add_one/:num").get(add_one);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/add_one/").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 404);
}
