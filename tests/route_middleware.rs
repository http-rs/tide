use futures::future::BoxFuture;
use http_service::Body;
use http_service_mock::make_server;
use tide::Middleware;

struct TestMiddleware(&'static str, &'static str);

impl TestMiddleware {
    fn with_header_name(name: &'static str, value: &'static str) -> Self {
        Self(name, value)
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for TestMiddleware {
    fn handle<'a>(
        &'a self,
        req: tide::Request<State>,
        next: tide::Next<'a, State>,
    ) -> BoxFuture<'a, tide::Response> {
        Box::pin(async move {
            let res = next.run(req).await;
            res.set_header(self.0, self.1)
        })
    }
}

async fn echo_path<State>(req: tide::Request<State>) -> String {
    req.uri().path().to_string()
}

#[test]
fn route_middleware() {
    let mut app = tide::new();
    let mut foo_route = app.at("/foo");
    foo_route // /foo
        .middleware(TestMiddleware::with_header_name("X-Foo", "foo"))
        .get(echo_path);
    foo_route
        .at("/bar") // nested, /foo/bar
        .middleware(TestMiddleware::with_header_name("X-Bar", "bar"))
        .get(echo_path);
    foo_route // /foo
        .post(echo_path)
        .reset_middleware()
        .put(echo_path);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/foo").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.headers().get("X-Foo"), Some(&"foo".parse().unwrap()));

    let req = http::Request::post("/foo").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.headers().get("X-Foo"), Some(&"foo".parse().unwrap()));

    let req = http::Request::put("/foo").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.headers().get("X-Foo"), None);

    let req = http::Request::get("/foo/bar").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.headers().get("X-Foo"), Some(&"foo".parse().unwrap()));
    assert_eq!(res.headers().get("X-Bar"), Some(&"bar".parse().unwrap()));
}

#[test]
fn subroute_not_nested() {
    let mut app = tide::new();
    app.at("/parent") // /parent
        .middleware(TestMiddleware::with_header_name("X-Parent", "Parent"))
        .get(echo_path);
    app.at("/parent/child") // /parent/child, not nested
        .middleware(TestMiddleware::with_header_name("X-Child", "child"))
        .get(echo_path);
    let mut server = make_server(app.into_http_service()).unwrap();

    let req = http::Request::get("/parent/child")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.headers().get("X-Parent"), None);
    assert_eq!(
        res.headers().get("X-Child"),
        Some(&"child".parse().unwrap())
    );
}
