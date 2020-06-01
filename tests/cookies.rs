use async_std::prelude::*;
use tide::http::cookies::Cookie;
use tide::http::headers::{COOKIE, SET_COOKIE};

use tide::{Request, Response, Server, StatusCode};

static COOKIE_NAME: &str = "testCookie";

async fn retrieve_cookie(cx: Request<()>) -> tide::Result<String> {
    Ok(format!(
        "{} and also {}",
        cx.cookie(COOKIE_NAME).unwrap().value(),
        cx.cookie("secondTestCookie").unwrap().value()
    ))
}

async fn insert_cookie(_req: Request<()>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.insert_cookie(Cookie::new(COOKIE_NAME, "NewCookieValue"));
    Ok(res)
}

async fn remove_cookie(_req: Request<()>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.remove_cookie(Cookie::named(COOKIE_NAME));
    Ok(res)
}

async fn set_multiple_cookie(_req: Request<()>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.insert_cookie(Cookie::new("C1", "V1"));
    res.insert_cookie(Cookie::new("C2", "V2"));
    Ok(res)
}

fn app() -> crate::Server<()> {
    let mut app = tide::new();

    app.at("/get").get(retrieve_cookie);
    app.at("/set").get(insert_cookie);
    app.at("/remove").get(remove_cookie);
    app.at("/multi").get(set_multiple_cookie);
    app
}

async fn make_request(endpoint: &str) -> http_types::Response {
    let app = app();
    let mut req = http_types::Request::new(
        http_types::Method::Get,
        http_types::url::Url::parse("http://example.com")
            .unwrap()
            .join(endpoint)
            .unwrap(),
    );

    req.insert_header(
        COOKIE,
        "testCookie=RequestCookieValue; secondTestCookie=Other%3BCookie%20Value",
    );

    let res: tide::http::Response = app.respond(req).await.unwrap();
    res
}

#[async_std::test]
async fn successfully_retrieve_request_cookie() {
    let mut res = make_request("/get").await;
    assert_eq!(res.status(), StatusCode::Ok);

    let mut buffer = Vec::new();
    res.read_to_end(&mut buffer).await.unwrap();
    let body = String::from_utf8(buffer).unwrap();

    assert_eq!(&body, "RequestCookieValue and also Other;Cookie Value");
}

#[async_std::test]
async fn successfully_insert_cookie() {
    let res = make_request("/set").await;
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(res[SET_COOKIE], "testCookie=NewCookieValue");
}

#[async_std::test]
async fn successfully_remove_cookie() {
    let res = make_request("/remove").await;
    assert_eq!(res.status(), StatusCode::Ok);
    let test_cookie_header = res[SET_COOKIE].last().as_str();
    assert!(test_cookie_header.starts_with("testCookie=;"));
    let cookie = Cookie::parse_encoded(test_cookie_header).unwrap();
    assert_eq!(cookie.name(), COOKIE_NAME);
    assert_eq!(cookie.value(), "");
    assert_eq!(cookie.http_only(), None);
    assert_eq!(cookie.max_age().unwrap().whole_nanoseconds(), 0);
}

#[async_std::test]
async fn successfully_set_multiple_cookies() {
    let res = make_request("/multi").await;
    assert_eq!(res.status(), StatusCode::Ok);
    let cookie_header = res[SET_COOKIE].clone();
    let cookies = cookie_header.iter().collect::<Vec<_>>();
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
