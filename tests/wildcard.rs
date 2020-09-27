mod test_utils;
use test_utils::ServerTestingExt;
use tide::{Error, Request, StatusCode};

async fn add_one(req: Request<()>) -> Result<String, tide::Error> {
    let num: i64 = req
        .param("num")?
        .parse()
        .map_err(|err| Error::new(StatusCode::BadRequest, err))?;
    Ok((num + 1).to_string())
}

async fn add_two(req: Request<()>) -> Result<String, tide::Error> {
    let one: i64 = req
        .param("one")?
        .parse()
        .map_err(|err| Error::new(StatusCode::BadRequest, err))?;
    let two: i64 = req
        .param("two")?
        .parse()
        .map_err(|err| Error::new(StatusCode::BadRequest, err))?;
    Ok((one + two).to_string())
}

async fn echo_path(req: Request<()>) -> Result<String, tide::Error> {
    match req.param("path") {
        Ok(path) => Ok(path.into()),
        Err(mut err) => {
            err.set_status(StatusCode::BadRequest);
            Err(err)
        }
    }
}

#[async_std::test]
async fn wildcard() {
    let mut app = tide::Server::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(app.get_body("/add_one/3").await, "4");
    assert_eq!(app.get_body("/add_one/-7").await, "-6");
}

#[async_std::test]
async fn invalid_segment_error() {
    let mut app = tide::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(app.get("/add_one/a").await.status(), StatusCode::BadRequest);
}

#[async_std::test]
async fn not_found_error() {
    let mut app = tide::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(app.get("/add_one/").await.status(), StatusCode::NotFound);
}

#[async_std::test]
async fn wild_path() {
    let mut app = tide::new();
    app.at("/echo/*path").get(echo_path);
    assert_eq!(app.get_body("/echo/some_path").await, "some_path");
    assert_eq!(
        app.get_body("/echo/multi/segment/path").await,
        "multi/segment/path"
    );
    assert_eq!(app.get("/echo/").await.status(), StatusCode::NotFound);
}

#[async_std::test]
async fn multi_wildcard() {
    let mut app = tide::new();
    app.at("/add_two/:one/:two/").get(add_two);
    assert_eq!(app.get_body("/add_two/1/2/").await, "3");
    assert_eq!(app.get_body("/add_two/-1/2/").await, "1");
    assert_eq!(app.get("/add_two/1").await.status(), StatusCode::NotFound);
}

#[async_std::test]
async fn wild_last_segment() {
    let mut app = tide::new();
    app.at("/echo/:path/*").get(echo_path);
    assert_eq!(app.get_body("/echo/one/two").await, "one");
    assert_eq!(app.get_body("/echo/one/two/three/four").await, "one");
}

#[async_std::test]
async fn invalid_wildcard() {
    let mut app = tide::new();
    app.at("/echo/*path/:one/").get(echo_path);
    assert_eq!(
        app.get("/echo/one/two").await.status(),
        StatusCode::NotFound
    );
}

#[async_std::test]
async fn nameless_wildcard() {
    let mut app = tide::Server::new();
    app.at("/echo/:").get(|_| async { Ok("") });
    assert_eq!(
        app.get("/echo/one/two").await.status(),
        StatusCode::NotFound
    );
    assert_eq!(app.get("/echo/one").await.status(), StatusCode::Ok);
}

#[async_std::test]
async fn nameless_internal_wildcard() {
    let mut app = tide::new();
    app.at("/echo/:/:path").get(echo_path);
    assert_eq!(app.get("/echo/one").await.status(), StatusCode::NotFound);
    assert_eq!(app.get_body("/echo/one/two").await, "two");
}

#[async_std::test]
async fn nameless_internal_wildcard2() {
    let mut app = tide::new();
    app.at("/echo/:/:path").get(|req: Request<()>| async move {
        assert_eq!(req.param("path")?, "two");
        Ok("")
    });

    app.get("/echo/one/two").await;
}

#[async_std::test]
async fn ambiguous_router_wildcard_vs_star() {
    let mut app = tide::new();
    app.at("/:one/:two").get(|_| async { Ok("one/two") });
    app.at("/posts/*").get(|_| async { Ok("posts/*") });
    assert_eq!(app.get_body("/posts/10").await, "posts/*");
}
