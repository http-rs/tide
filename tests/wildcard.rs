use tide::{testing::TestingExt, Request, StatusCode};
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
async fn wildcard() -> tide::Result<()> {
    let mut app = tide::Server::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(app.get("/add_one/3").recv_string().await?, "4");
    assert_eq!(app.get("/add_one/-7").recv_string().await?, "-6");
    Ok(())
}

#[async_std::test]
async fn invalid_segment_error() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(
        app.get("/add_one/a").await?.status(),
        StatusCode::BadRequest
    );
    Ok(())
}

#[async_std::test]
async fn not_found_error() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/add_one/:num").get(add_one);
    assert_eq!(app.get("/add_one/").await?.status(), StatusCode::NotFound);
    Ok(())
}

#[async_std::test]
async fn wild_path() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/echo/*path").get(echo_path);
    assert_eq!(app.get("/echo/some_path").recv_string().await?, "some_path");
    assert_eq!(
        app.get("/echo/multi/segment/path").recv_string().await?,
        "multi/segment/path"
    );
    assert_eq!(app.get("/echo/").await?.status(), StatusCode::NotFound);
    Ok(())
}

#[async_std::test]
async fn multi_wildcard() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/add_two/:one/:two/").get(add_two);
    assert_eq!(app.get("/add_two/1/2/").recv_string().await?, "3");
    assert_eq!(app.get("/add_two/-1/2/").recv_string().await?, "1");
    assert_eq!(app.get("/add_two/1").await?.status(), StatusCode::NotFound);
    Ok(())
}

#[async_std::test]
async fn wild_last_segment() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/echo/:path/*").get(echo_path);
    assert_eq!(app.get("/echo/one/two").recv_string().await?, "one");
    assert_eq!(
        app.get("/echo/one/two/three/four").recv_string().await?,
        "one"
    );
    Ok(())
}

#[async_std::test]
async fn invalid_wildcard() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/echo/*path/:one/").get(echo_path);
    assert_eq!(
        app.get("/echo/one/two").await?.status(),
        StatusCode::NotFound
    );
    Ok(())
}

#[async_std::test]
async fn nameless_wildcard() -> tide::Result<()> {
    let mut app = tide::Server::new();
    app.at("/echo/:").get(|_| async { Ok("") });
    assert_eq!(
        app.get("/echo/one/two").await?.status(),
        StatusCode::NotFound
    );
    assert_eq!(app.get("/echo/one").await?.status(), StatusCode::Ok);
    Ok(())
}

#[async_std::test]
async fn nameless_internal_wildcard() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/echo/:/:path").get(echo_path);
    assert_eq!(app.get("/echo/one").await?.status(), StatusCode::NotFound);
    assert_eq!(app.get("/echo/one/two").recv_string().await?, "two");
    Ok(())
}

#[async_std::test]
async fn nameless_internal_wildcard2() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/echo/:/:path").get(|req: Request<()>| async move {
        assert_eq!(req.param::<String>("path")?, "two");
        Ok("")
    });

    assert!(app.get("/echo/one/two").await?.status().is_success());
    Ok(())
}

#[async_std::test]
async fn ambiguous_router_wildcard_vs_star() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/:one/:two").get(|_| async { Ok("one/two") });
    app.at("/posts/*").get(|_| async { Ok("posts/*") });
    assert_eq!(app.get("/posts/10").recv_string().await?, "posts/*");
    Ok(())
}
