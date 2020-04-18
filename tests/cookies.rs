use cookie::Cookie;
use futures::executor::block_on;
use futures::AsyncReadExt;
use http_service_mock::make_server;
use http_types::StatusCode;

use tide::{Request, Response, Server};

static COOKIE_NAME: &str = "testCookie";

async fn retrieve_cookie(cx: Request<()>) -> tide::Result<String> {
    Ok(cx.cookie(COOKIE_NAME).unwrap().value().to_string())
}

async fn set_cookie(_req: Request<()>) -> tide::Result<Response> {
    let mut res = Response::new(StatusCode::Ok);
    res.set_cookie(Cookie::new(COOKIE_NAME, "NewCookieValue"));
    Ok(res)
}

async fn remove_cookie(_req: Request<()>) -> tide::Result<Response> {
    let mut res = Response::new(StatusCode::Ok);
    res.remove_cookie(Cookie::named(COOKIE_NAME));
    Ok(res)
}

async fn set_multiple_cookie(_req: Request<()>) -> tide::Result<Response> {
    let mut res = Response::new(StatusCode::Ok);
    res.set_cookie(Cookie::new("C1", "V1"));
    res.set_cookie(Cookie::new("C2", "V2"));
    Ok(res)
}

fn app() -> crate::Server<()> {
    let mut app = tide::new();

    app.at("/get").get(retrieve_cookie);
    app.at("/set").get(set_cookie);
    app.at("/remove").get(remove_cookie);
    app.at("/multi").get(set_multiple_cookie);
    app
}

fn make_request(endpoint: &str) -> http_types::Response {
    let app = app();
    let mut server = make_server(app.into_http_service()).unwrap();
    let mut req = http_types::Request::new(
        http_types::Method::Get,
        format!("http://example.com{}", endpoint).parse().unwrap(),
    );
    req.insert_header(http_types::headers::COOKIE, "testCookie=RequestCookieValue")
        .unwrap();

    server.simulate(req).unwrap()
}

#[test]
fn successfully_retrieve_request_cookie() {
    let mut res = make_request("/get");
    assert_eq!(res.status(), StatusCode::Ok);

    let body = block_on(async move {
        let mut buffer = Vec::new();
        res.read_to_end(&mut buffer).await.unwrap();
        buffer
    });

    assert_eq!(&*body, &*b"RequestCookieValue");
}

#[test]
fn successfully_set_cookie() {
    let res = make_request("/set");
    assert_eq!(res.status(), StatusCode::Ok);
    let test_cookie_header = res.header(&http_types::headers::SET_COOKIE).unwrap()[0].as_str();
    assert_eq!(test_cookie_header, "testCookie=NewCookieValue");
}

#[test]
fn successfully_remove_cookie() {
    let res = make_request("/remove");
    assert_eq!(res.status(), StatusCode::Ok);
    let test_cookie_header = res.header(&http_types::headers::SET_COOKIE).unwrap()[0].as_str();
    assert!(test_cookie_header.starts_with("testCookie=;"));
    let cookie = Cookie::parse_encoded(test_cookie_header).unwrap();
    assert_eq!(cookie.name(), COOKIE_NAME);
    assert_eq!(cookie.value(), "");
    assert_eq!(cookie.http_only(), None);
    assert_eq!(cookie.max_age().unwrap().num_nanoseconds(), Some(0));
}

#[test]
fn successfully_set_multiple_cookies() {
    let res = make_request("/multi");
    assert_eq!(res.status(), StatusCode::Ok);
    let cookie_header = res.header(&http_types::headers::SET_COOKIE);
    let cookies = cookie_header.unwrap().iter().collect::<Vec<_>>();
    assert_eq!(cookies.len(), 2, "{:?}", &cookies);
    let cookie1 = cookies[0].as_str();
    let cookie2 = cookies[1].as_str();

    // Headers can be out of order
    if cookie1.starts_with("C1") {
        assert_eq!(cookie1, "C1=V1");
        assert_eq!(cookie2, "C2=V2");
    } else {
        assert_eq!(cookie2, "C1=V1");
        assert_eq!(cookie1, "C2=V2");
    }
}
