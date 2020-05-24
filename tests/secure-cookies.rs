use std::sync::Arc;

use tide::http::cookies::{Cookie, Key};
use tide::http::headers::{COOKIE, SET_COOKIE};
use tide::{Request, Response, Server, StatusCode};

static SIGNED_COOKIE_NAME: &str = "signedTestCookie";
static PRIVATE_COOKIE_NAME: &str = "privateTestCookie";
const COOKIE_VALUE: &str = "CookieValue";

async fn retrieve_plain_cookie(req: Request<Arc<Key>>) -> tide::Result<String> {
    let name = req.param::<String>("name").unwrap();
    Ok(req.cookie(&name).unwrap().value().to_string())
}

async fn retrieve_signed_cookie(req: Request<Arc<Key>>) -> tide::Result<String> {
    let name = req.param::<String>("name").unwrap();
    Ok(req
        .signed_cookie(&req.state(), &name)
        .unwrap()
        .value()
        .to_string())
}

async fn retrieve_private_cookie(req: Request<Arc<Key>>) -> tide::Result<String> {
    let name = req.param::<String>("name").unwrap();
    Ok(req
        .private_cookie(&req.state(), &name)
        .unwrap()
        .value()
        .to_string())
}

async fn set_signed_cookie(req: Request<Arc<Key>>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.set_signed_cookie(
        req.state().clone(),
        Cookie::new(SIGNED_COOKIE_NAME, COOKIE_VALUE),
    );
    Ok(res)
}

async fn set_private_cookie(req: Request<Arc<Key>>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.set_private_cookie(
        req.state().clone(),
        Cookie::new(PRIVATE_COOKIE_NAME, COOKIE_VALUE),
    );
    Ok(res)
}

async fn remove_signed_cookie(_req: Request<Arc<Key>>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.remove_cookie(Cookie::named(SIGNED_COOKIE_NAME));
    Ok(res)
}

async fn remove_private_cookie(_req: Request<Arc<Key>>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.remove_cookie(Cookie::named(PRIVATE_COOKIE_NAME));
    Ok(res)
}

fn app() -> crate::Server<Arc<Key>> {
    let session_key = Arc::new(Key::derive_from(&[0u8; 32]));
    let mut app = tide::with_state(session_key);

    app.at("/get/plain/:name").get(retrieve_plain_cookie);
    app.at("/get/signed/:name").get(retrieve_signed_cookie);
    app.at("/get/private/:name").get(retrieve_private_cookie);

    app.at("/set/signed").get(set_signed_cookie);
    app.at("/set/private").get(set_private_cookie);

    app.at("/remove/signed").get(remove_signed_cookie);
    app.at("/remove/private").get(remove_private_cookie);

    app
}

async fn make_request(endpoint: &str, cookie_header: Option<&str>) -> http_types::Response {
    let app = app();
    let mut req = http_types::Request::new(
        http_types::Method::Get,
        http_types::url::Url::parse("http://example.com")
            .unwrap()
            .join(endpoint)
            .unwrap(),
    );

    if let Some(value) = cookie_header {
        req.insert_header(COOKIE, value);
    };

    let res: tide::http::Response = app.respond(req).await.unwrap();
    res
}

#[async_std::test]
async fn successfully_set_signed_cookie() {
    let res = make_request("/set/signed", None).await;
    assert_eq!(res.status(), StatusCode::Ok);
    assert!(res[SET_COOKIE].as_str().starts_with(SIGNED_COOKIE_NAME));
    assert_ne!(res[SET_COOKIE], COOKIE_VALUE);
}

#[async_std::test]
async fn successfully_set_private_cookie() {
    let res = make_request("/set/private", None).await;
    assert_eq!(res.status(), StatusCode::Ok);
    assert!(res[SET_COOKIE].as_str().starts_with(PRIVATE_COOKIE_NAME));
    assert_ne!(res[SET_COOKIE], COOKIE_VALUE);
}

#[async_std::test]
async fn successfully_set_and_remove_signed_cookie() {
    let res = make_request("/set/signed", None).await;
    let cookie = res[SET_COOKIE].last().as_str();

    let res = make_request("/remove/signed", Some(cookie)).await;
    assert_eq!(res.status(), StatusCode::Ok);

    let test_cookie_header = res[SET_COOKIE].last().as_str();
    assert!(test_cookie_header.starts_with(&format!("{}=;", SIGNED_COOKIE_NAME)));
    let cookie = Cookie::parse_encoded(test_cookie_header).unwrap();
    assert_eq!(cookie.name(), SIGNED_COOKIE_NAME);
    assert_eq!(cookie.value(), "");
    assert_eq!(cookie.http_only(), None);
    assert_eq!(cookie.max_age().unwrap().whole_nanoseconds(), 0);
}

#[async_std::test]
async fn successfully_set_and_remove_private_cookie() {
    let res = make_request("/set/private", None).await;
    let cookie = res[SET_COOKIE].last().as_str();

    let res = make_request("/remove/private", Some(cookie)).await;
    assert_eq!(res.status(), StatusCode::Ok);

    let test_cookie_header = res[SET_COOKIE].last().as_str();
    assert!(test_cookie_header.starts_with(&format!("{}=;", PRIVATE_COOKIE_NAME)));
    let cookie = Cookie::parse_encoded(test_cookie_header).unwrap();
    assert_eq!(cookie.name(), PRIVATE_COOKIE_NAME);
    assert_eq!(cookie.value(), "");
    assert_eq!(cookie.http_only(), None);
    assert_eq!(cookie.max_age().unwrap().whole_nanoseconds(), 0);
}
