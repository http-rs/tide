use async_std::io::prelude::ReadExt;
use futures::executor::block_on;
use http_service::Body;
use http_service_mock::{make_server, TestBackend};
use serde::Deserialize;
use tide::{server::Service, IntoResponse, Request, Response, Server};

#[derive(Deserialize)]
struct Params {
    msg: String,
}

#[derive(Deserialize)]
struct OptionalParams {
    _msg: Option<String>,
    _time: Option<u64>,
}

async fn handler(cx: Request<()>) -> Response {
    let p = cx.query::<Params>();
    match p {
        Ok(params) => params.msg.into_response(),
        Err(_) => Response::new(400),
    }
}

async fn optional_handler(cx: Request<()>) -> Response {
    let p = cx.query::<OptionalParams>();
    match p {
        Ok(_) => Response::new(200),
        Err(_) => Response::new(400),
    }
}

fn get_server() -> TestBackend<Service<()>> {
    let mut app = Server::new();
    app.at("/").get(handler);
    app.at("/optional").get(optional_handler);
    make_server(app.into_http_service()).unwrap()
}

#[test]
fn successfully_deserialize_query() {
    let mut server = get_server();
    let req = http::Request::get("/?msg=Hello")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let mut body = String::new();
    block_on(res.into_body().read_to_string(&mut body)).unwrap();
    assert_eq!(body, "Hello");
}

#[test]
fn unsuccessfully_deserialize_query() {
    let mut server = get_server();
    let req = http::Request::get("/").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 400);

    let mut body = String::new();
    block_on(res.into_body().read_to_string(&mut body)).unwrap();
    // assert_eq!(body, "failed with reason: missing field `msg`");
}

#[test]
fn malformatted_query() {
    let mut server = get_server();
    let req = http::Request::get("/?error=should_fail")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 400);

    let mut body = String::new();
    block_on(res.into_body().read_to_string(&mut body)).unwrap();
    // assert_eq!(body, "failed with reason: missing field `msg`");
}

#[test]
fn empty_query_string_for_struct_with_no_required_fields() {
    let mut server = get_server();
    let req = http::Request::get("/optional").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
}
