use cookie::Cookie;
use futures::executor::block_on;
use futures::AsyncReadExt;
use http_service::Body;
use http_service_mock::make_server;

use tide::{middleware::CookiesMiddleware, Request, Response, Server};

static COOKIE_NAME: &str = "testCookie";

async fn retrieve_cookie(cx: Request<()>) -> String {
    cx.cookie(COOKIE_NAME).unwrap().unwrap().value().to_string()
}

async fn set_cookie(mut cx: Request<()>) -> Response {
    cx.set_cookie(Cookie::new(COOKIE_NAME, "NewCookieValue"))
        .unwrap();
    Response::new(200)
}

async fn remove_cookie(mut cx: Request<()>) -> Response {
    cx.remove_cookie(Cookie::named(COOKIE_NAME)).unwrap();
    Response::new(200)
}

async fn set_multiple_cookie(mut cx: Request<()>) -> Response {
    cx.set_cookie(Cookie::new("C1", "V1")).unwrap();
    cx.set_cookie(Cookie::new("C2", "V2")).unwrap();
    Response::new(200)
}

fn app() -> crate::Server<()> {
    let mut app = tide::new();
    app.middleware(CookiesMiddleware::new());

    app.at("/get").get(retrieve_cookie);
    app.at("/set").get(set_cookie);
    app.at("/remove").get(remove_cookie);
    app.at("/multi").get(set_multiple_cookie);
    app
}

fn make_request(endpoint: &str) -> http::response::Response<http_service::Body> {
    let app = app();
    let mut server = make_server(app.into_http_service()).unwrap();
    let req = http::Request::get(endpoint)
        .header(http::header::COOKIE, "testCookie=RequestCookieValue")
        .body(Body::empty())
        .unwrap();
    server.simulate(req).unwrap()
}

#[test]
fn successfully_retrieve_request_cookie() {
    let mut res = make_request("/get");
    assert_eq!(res.status(), 200);

    let body = block_on(async move {
        let mut buffer = Vec::new();
        res.body_mut().read_to_end(&mut buffer).await.unwrap();
        buffer
    });

    assert_eq!(&*body, &*b"RequestCookieValue");
}

#[test]
fn successfully_set_cookie() {
    let res = make_request("/set");
    assert_eq!(res.status(), 200);
    let test_cookie_header = res.headers().get(http::header::SET_COOKIE).unwrap();
    assert_eq!(
        test_cookie_header.to_str().unwrap(),
        "testCookie=NewCookieValue"
    );
}

#[test]
fn successfully_remove_cookie() {
    let res = make_request("/remove");
    assert_eq!(res.status(), 200);
    let test_cookie_header = res.headers().get(http::header::SET_COOKIE).unwrap();
    assert!(test_cookie_header
        .to_str()
        .unwrap()
        .starts_with("testCookie=;"));
    let cookie = Cookie::parse_encoded(test_cookie_header.to_str().unwrap()).unwrap();
    assert_eq!(cookie.name(), COOKIE_NAME);
    assert_eq!(cookie.value(), "");
    assert_eq!(cookie.http_only(), None);
    assert_eq!(cookie.max_age().unwrap().num_nanoseconds(), Some(0));
}

#[test]
fn successfully_set_multiple_cookies() {
    let res = make_request("/multi");
    assert_eq!(res.status(), 200);
    let cookie_header = res.headers().get_all(http::header::SET_COOKIE);
    let mut iter = cookie_header.iter();

    let cookie1 = iter.next().unwrap();
    let cookie2 = iter.next().unwrap();

    //Headers can be out of order
    if cookie1.to_str().unwrap().starts_with("C1") {
        assert_eq!(cookie1, "C1=V1");
        assert_eq!(cookie2, "C2=V2");
    } else {
        assert_eq!(cookie2, "C1=V1");
        assert_eq!(cookie1, "C2=V2");
    }

    assert!(iter.next().is_none());
}
