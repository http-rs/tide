use async_std::io::prelude::*;
use futures::executor::block_on;
use futures::future::BoxFuture;
use http_service::Body;
use http_service_mock::make_server;

#[test]
fn nested() {
    let mut inner = tide::new();
    inner.at("/foo").get(|_| async { "foo" });
    inner.at("/bar").get(|_| async { "bar" });

    let mut outer = tide::new();
    // Nest the inner app on /foo
    outer.at("/foo").nest_service(inner);

    let mut server = make_server(outer.into_http_service()).unwrap();

    let mut buf = Vec::new();
    let req = http::Request::get("/foo/foo").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"foo");

    buf.clear();
    let req = http::Request::get("/foo/bar").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"bar");
}

#[test]
fn nested_middleware() {
    let echo_path = |req: tide::Request<()>| async move { req.uri().path().to_string() };
    fn test_middleware(
        req: tide::Request<()>,
        next: tide::Next<'_, ()>,
    ) -> BoxFuture<'_, tide::Response> {
        Box::pin(async move {
            let res = next.run(req).await;
            res.set_header("X-Tide-Test", "1")
        })
    }

    let mut app = tide::new();
    app.at("/foo").nest(|route| {
        let mut app = tide::new();
        app.middleware(test_middleware);
        app.at("/echo").get(echo_path);
        app.at("/:foo/bar").strip_prefix().get(echo_path);
        route.nest_service(app);
    });
    app.at("/bar").get(echo_path);

    let mut server = make_server(app.into_http_service()).unwrap();

    let mut buf = Vec::new();
    let req = http::Request::get("/foo/echo").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(
        res.headers().get("X-Tide-Test"),
        Some(&"1".parse().unwrap())
    );
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"/echo");

    buf.clear();
    let req = http::Request::get("/foo/x/bar")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(
        res.headers().get("X-Tide-Test"),
        Some(&"1".parse().unwrap())
    );
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"/");

    buf.clear();
    let req = http::Request::get("/bar").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.headers().get("X-Tide-Test"), None);
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"/bar");
}
