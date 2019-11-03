use futures::executor::block_on;
use http_service::Body;
use http_service_mock::make_server;
use tide::{error::ResultExt, Context};

async fn add_one(cx: Context<()>) -> Result<String, tide::Error> {
    let num: i64 = cx.param("num").client_err()?;
    Ok((num + 1).to_string())
}

async fn add_two(cx: Context<()>) -> Result<String, tide::Error> {
    let one: i64 = cx.param("one").client_err()?;
    let two: i64 = cx.param("two").client_err()?;
    Ok((one + two).to_string())
}

async fn echo_path(cx: Context<()>) -> Result<String, tide::Error> {
    let path: String = cx.param("path").client_err()?;
    Ok(path)
}

async fn echo_empty(cx: Context<()>) -> Result<String, tide::Error> {
    let path: String = cx.param("").client_err()?;
    Ok(path)
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

#[test]
fn wildpath() {
    let mut app = tide::App::new();
    app.at("/echo/*path").get(echo_path);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/echo/some_path")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"some_path");

    let req = http::Request::get("/echo/multi/segment/path")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"multi/segment/path");

    let req = http::Request::get("/echo/").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 404);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"");
}

#[test]
fn multi_wildcard() {
    let mut app = tide::App::new();
    app.at("/add_two/:one/:two/").get(add_two);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/add_two/1/2/")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"3");

    let req = http::Request::get("/add_two/-1/2/")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"1");
    let req = http::Request::get("/add_two/1")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 404);
}

#[test]
fn wild_last_segment() {
    let mut app = tide::App::new();
    app.at("/echo/:path/*").get(echo_path);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/echo/one/two")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"one");

    let req = http::Request::get("/echo/one/two/three/four")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"one");
}

#[test]
fn invalid_wildcard() {
    let mut app = tide::App::new();
    app.at("/echo/*path/:one/").get(echo_path);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/echo/one/two")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 404);
}

#[test]
fn nameless_wildcard() {
    let mut app = tide::App::new();
    app.at("/echo/:").get(|_| async move { "" });

    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/echo/one/two")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 404);

    let req = http::Request::get("/echo/one").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
}

#[test]
fn nameless_internal_wildcard() {
    let mut app = tide::App::new();
    app.at("/echo/:/:path").get(echo_path);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/echo/one").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 404);

    let req = http::Request::get("/echo/one/two")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"two");

    let req = http::Request::get("/echo/one/two")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"two");
}

#[test]
fn nameless_internal_wildcard2() {
    let mut app = tide::App::new();
    app.at("/echo/:/:path").get(echo_empty);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/echo/one/two")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    let body = block_on(res.into_body().into_vec()).unwrap();
    assert_eq!(&*body, &*b"one");
}
