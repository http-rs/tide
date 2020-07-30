mod test_utils;
use test_utils::ServerTestingExt;

#[async_std::test]
async fn nested() {
    let mut inner = tide::new();
    inner.at("/foo").get(|_| async { Ok("foo") });
    inner.at("/bar").get(|_| async { Ok("bar") });

    let mut outer = tide::new();
    // Nest the inner app on /foo
    outer.at("/foo").nest(inner);

    assert_eq!(outer.get_body("/foo/foo").await, "foo");
    assert_eq!(outer.get_body("/foo/bar").await, "bar");
}

#[async_std::test]
async fn nested_middleware() {
    let echo_path = |req: tide::Request<()>| async move { Ok(req.url().path().to_string()) };
    let mut app = tide::new();
    let mut inner_app = tide::new();
    inner_app.with(tide::utils::After(|mut res: tide::Response| async move {
        res.insert_header("x-tide-test", "1");
        Ok(res)
    }));
    inner_app.at("/echo").get(echo_path);
    inner_app.at("/:foo/bar").strip_prefix().get(echo_path);
    app.at("/foo").nest(inner_app);
    app.at("/bar").get(echo_path);

    let mut res = app.get("/foo/echo").await;
    assert_eq!(res["X-Tide-Test"], "1");
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "/echo");

    let mut res = app.get("/foo/x/bar").await;
    assert_eq!(res["X-Tide-Test"], "1");
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "/");

    let mut res = app.get("/bar").await;
    assert!(res.header("X-Tide-Test").is_none());
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "/bar");
}

#[async_std::test]
async fn nested_with_different_state() {
    let mut outer = tide::new();
    let mut inner = tide::with_state(42);
    inner.at("/").get(|req: tide::Request<i32>| async move {
        let num = req.state();
        Ok(format!("the number is {}", num))
    });
    outer.at("/").get(|_| async { Ok("Hello, world!") });
    outer.at("/foo").nest(inner);

    assert_eq!(outer.get_body("/foo").await, "the number is 42");
    assert_eq!(outer.get_body("/").await, "Hello, world!");
}
