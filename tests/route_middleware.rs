use async_std::io::prelude::*;
use futures::executor::block_on;
use futures::future::BoxFuture;
use http_service::Body;
use http_service_mock::make_server;
use tide::Middleware;

struct TestMiddleware(&'static str);

impl<State: Send + Sync + 'static> Middleware<State> for TestMiddleware {
    fn handle<'a>(
        &'a self,
        req: tide::Request<State>,
        next: tide::Next<'a, State>,
    ) -> BoxFuture<'a, tide::Response> {
        Box::pin(async move {
            let res = next.run(req).await;
            res.set_header("X-Tide-Test", self.0)
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
    foo_route.middleware(TestMiddleware("foo")).get(echo_path);
    foo_route
        .at("/bar")
        .middleware(TestMiddleware("bar"))
        .get(echo_path);
    foo_route.post(echo_path).reset_middleware().put(echo_path);
    let mut server = make_server(app.into_http_service()).unwrap();

    let mut buf = Vec::new();
    let req = http::Request::get("/foo").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(
        res.headers().get("X-Tide-Test"),
        Some(&"foo".parse().unwrap())
    );
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"/foo");

    buf.clear();
    let req = http::Request::post("/foo").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(
        res.headers().get("X-Tide-Test"),
        Some(&"foo".parse().unwrap())
    );
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"/foo");

    buf.clear();
    let req = http::Request::put("/foo").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(res.headers().get("X-Tide-Test"), None);
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"/foo");

    buf.clear();
    let req = http::Request::get("/foo/bar").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();
    assert_eq!(
        res.headers().get("X-Tide-Test"),
        Some(&"bar".parse().unwrap())
    );
    assert_eq!(res.status(), 200);
    block_on(res.into_body().read_to_end(&mut buf)).unwrap();
    assert_eq!(&*buf, &*b"/foo/bar");
}
