use http_types::{Method, StatusCode, Url};
use tide::{http, Request};

async fn add_one(cx: Request<()>) -> Result<String, tide::Error> {
    return match cx.param::<i64>("num") {
        Ok(num) => Ok((num + 1).to_string()),
        Err(err) => Err(tide::Error::new(StatusCode::BadRequest, err)),
    };
}

async fn add_two(cx: Request<()>) -> Result<String, tide::Error> {
    let one;
    match cx.param::<i64>("one") {
        Ok(num) => one = num,
        Err(err) => return Err(tide::Error::new(StatusCode::BadRequest, err)),
    };
    let two;
    match cx.param::<i64>("two") {
        Ok(num) => two = num,
        Err(err) => return Err(tide::Error::new(StatusCode::BadRequest, err)),
    };
    Ok((one + two).to_string())
}

async fn echo_path(cx: Request<()>) -> Result<String, tide::Error> {
    return match cx.param::<String>("path") {
        Ok(path) => Ok(path),
        Err(err) => Err(tide::Error::new(StatusCode::BadRequest, err)),
    };
}

async fn echo_empty(cx: Request<()>) -> Result<String, tide::Error> {
    return match cx.param::<String>("") {
        Ok(path) => Ok(path),
        Err(err) => Err(tide::Error::new(StatusCode::BadRequest, err)),
    };
}

#[async_std::test]
async fn wildcard() {
    let mut app = tide::Server::new();
    app.at("/add_one/:num").get(add_one);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_one/3").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"4");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_one/-7").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"-6");
}

#[async_std::test]
async fn invalid_segment_error() {
    let mut app = tide::new();
    app.at("/add_one/:num").get(add_one);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_one/a").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BadRequest);
}

#[async_std::test]
async fn not_found_error() {
    let mut app = tide::new();
    app.at("/add_one/:num").get(add_one);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_one/").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NotFound);
}

#[async_std::test]
async fn wild_path() {
    let mut app = tide::new();
    app.at("/echo/*path").get(echo_path);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/some_path").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"some_path");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/multi/segment/path").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"multi/segment/path");

    let req = http::Request::new(Method::Get, Url::parse("http://localhost/echo/").unwrap());
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NotFound);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"");
}

#[async_std::test]
async fn multi_wildcard() {
    let mut app = tide::new();
    app.at("/add_two/:one/:two/").get(add_two);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_two/1/2/").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"3");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_two/-1/2/").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), 200);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"1");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_two/1").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NotFound);
}

#[async_std::test]
async fn wild_last_segment() {
    let mut app = tide::new();
    app.at("/echo/:path/*").get(echo_path);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"one");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two/three/four").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"one");
}

#[async_std::test]
async fn invalid_wildcard() {
    let mut app = tide::new();
    app.at("/echo/*path/:one/").get(echo_path);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NotFound);
}

#[async_std::test]
async fn nameless_wildcard() {
    let mut app = tide::Server::new();
    app.at("/echo/:").get(|_| async move { Ok("") });

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NotFound);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
}

#[async_std::test]
async fn nameless_internal_wildcard() {
    let mut app = tide::new();
    app.at("/echo/:/:path").get(echo_path);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NotFound);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"two");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"two");
}

#[async_std::test]
async fn nameless_internal_wildcard2() {
    let mut app = tide::new();
    app.at("/echo/:/:path").get(echo_empty);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    let body: String = res.take_body().into_string().await.unwrap();
    assert_eq!(body.as_bytes(), b"one");
}
