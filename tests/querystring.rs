use async_std::prelude::*;
use http_types::{url::Url, Method};
use serde::Deserialize;
use tide::{http, Request, Response, Server, StatusCode};

#[derive(Deserialize)]
struct Params {
    msg: String,
}

#[derive(Deserialize)]
struct OptionalParams {
    _msg: Option<String>,
    _time: Option<u64>,
}

async fn handler(cx: Request<()>) -> tide::Result {
    let p = cx.query::<Params>();
    match p {
        Ok(params) => Ok(params.msg.into()),
        Err(error) => Ok(err_to_res(error)),
    }
}

async fn optional_handler(cx: Request<()>) -> tide::Result {
    let p = cx.query::<OptionalParams>();
    match p {
        Ok(_) => Ok(Response::new(StatusCode::Ok)),
        Err(error) => Ok(err_to_res(error)),
    }
}

fn get_server() -> Server<()> {
    let mut app = Server::new();
    app.at("/").get(handler);
    app.at("/optional").get(optional_handler);
    app
}

#[async_std::test]
async fn successfully_deserialize_query() {
    let app = get_server();
    let req = http_types::Request::new(
        Method::Get,
        Url::parse("http://example.com/?msg=Hello").unwrap(),
    );

    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let mut body = String::new();
    res.read_to_string(&mut body).await.unwrap();
    assert_eq!(body, "Hello");
}

#[async_std::test]
async fn unsuccessfully_deserialize_query() {
    let app = get_server();
    let req = http_types::Request::new(Method::Get, Url::parse("http://example.com/").unwrap());
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), 400_u16);

    let mut body = String::new();
    res.read_to_string(&mut body).await.unwrap();
    assert_eq!(body, "failed with reason: missing field `msg`");
}

#[async_std::test]
async fn malformatted_query() {
    let app = get_server();
    let req = http_types::Request::new(
        Method::Get,
        Url::parse("http://example.com/?error=should_fail").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), 400);

    let mut body = String::new();
    res.read_to_string(&mut body).await.unwrap();
    assert_eq!(body, "failed with reason: missing field `msg`");
}

#[async_std::test]
async fn empty_query_string_for_struct_with_no_required_fields() {
    let app = get_server();
    let req = http_types::Request::new(
        Method::Get,
        Url::parse("http://example.com/optional").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
}

fn err_to_res(err: http_types::Error) -> crate::Response {
    Response::new(err.status())
        .set_header(
            http_types::headers::CONTENT_TYPE,
            "text/plain; charset=utf-8",
        )
        .body_string(err.to_string())
}
