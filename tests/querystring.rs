use async_std::io::prelude::ReadExt;
use futures::executor::block_on;
use http_service_mock::{make_server, TestBackend};
use http_types::StatusCode;
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
        Err(error) => Response::new(error.status()),
    }
}

async fn optional_handler(cx: Request<()>) -> Response {
    let p = cx.query::<OptionalParams>();
    match p {
        Ok(_) => Response::new(StatusCode::Ok),
        Err(error) => Response::new(error.status()),
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
    let req = http_types::Request::new(
        http_types::Method::Get,
        "http://example.com/?msg=Hello".parse().unwrap(),
    );

    let mut res = server.simulate(req).unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let mut body = String::new();
    block_on(res.read_to_string(&mut body)).unwrap();
    assert_eq!(body, "Hello");
}

#[test]
fn unsuccessfully_deserialize_query() {
    let mut server = get_server();
    let req = http_types::Request::new(
        http_types::Method::Get,
        "http://example.com/".parse().unwrap(),
    );
    let mut res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 400);

    let mut body = String::new();
    block_on(res.read_to_string(&mut body)).unwrap();
    assert_eq!(body, "failed with reason: missing field `msg`");
}

#[test]
fn malformatted_query() {
    let mut server = get_server();
    let req = http_types::Request::new(
        http_types::Method::Get,
        "http://example.com/?error=should_fail".parse().unwrap(),
    );
    let mut res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 400);

    let mut body = String::new();
    block_on(res.read_to_string(&mut body)).unwrap();
    assert_eq!(body, "failed with reason: missing field `msg`");
}

#[test]
fn empty_query_string_for_struct_with_no_required_fields() {
    let mut server = get_server();
    let req = http_types::Request::new(
        http_types::Method::Get,
        "http://example.com/optional".parse().unwrap(),
    );
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
}
