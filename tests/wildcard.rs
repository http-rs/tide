use http_types::{Method, StatusCode, Url};
use tide::{http, Request};

async fn add_one(req: Request<()>) -> Result<String, tide::Error> {
    match req.param::<i64>("num") {
        Ok(num) => Ok((num + 1).to_string()),
        Err(err) => Err(tide::Error::new(StatusCode::BadRequest, err)),
    }
}

async fn add_two(req: Request<()>) -> Result<String, tide::Error> {
    let one = req
        .param::<i64>("one")
        .map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?;
    let two = req
        .param::<i64>("two")
        .map_err(|err| tide::Error::new(StatusCode::BadRequest, err))?;
    Ok((one + two).to_string())
}

async fn echo_path(req: Request<()>) -> Result<String, tide::Error> {
    match req.param::<String>("path") {
        Ok(path) => Ok(path),
        Err(err) => Err(tide::Error::new(StatusCode::BadRequest, err)),
    }
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
    assert_eq!(&res.body_string().await.unwrap(), "4");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_one/-7").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(&res.body_string().await.unwrap(), "-6");
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
    assert_eq!(&res.body_string().await.unwrap(), "some_path");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/multi/segment/path").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(&res.body_string().await.unwrap(), "multi/segment/path");

    let req = http::Request::new(Method::Get, Url::parse("http://localhost/echo/").unwrap());
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NotFound);
    assert_eq!(&res.body_string().await.unwrap(), "");
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
    assert_eq!(&res.body_string().await.unwrap(), "3");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/add_two/-1/2/").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(&res.body_string().await.unwrap(), "1");

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
    assert_eq!(&res.body_string().await.unwrap(), "one");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two/three/four").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(&res.body_string().await.unwrap(), "one");
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
    app.at("/echo/:").get(|_| async { Ok("") });

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
    assert_eq!(&res.body_string().await.unwrap(), "two");

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two").unwrap(),
    );
    let mut res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(&res.body_string().await.unwrap(), "two");
}

#[async_std::test]
async fn nameless_internal_wildcard2() {
    let mut app = tide::new();
    app.at("/echo/:/:path").get(|req: Request<()>| async move {
        assert_eq!(req.param::<String>("path")?, "two");
        Ok("")
    });

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/echo/one/two").unwrap(),
    );
    let _: tide::Response = app.respond(req).await.unwrap();
}

#[async_std::test]
async fn ambiguous_router_wildcard_vs_star() {
    let mut app = tide::new();
    app.at("/:one/:two").get(|_| async { Ok("one/two") });
    app.at("/posts/*").get(|_| async { Ok("posts/*") });
    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/posts/10").unwrap(),
    );

    let mut response: http_types::Response = app.respond(req).await.unwrap();
    let body_string = response.body_string().await.unwrap();
    assert_eq!(body_string, "posts/*");
}
