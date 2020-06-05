use http_types::headers::HeaderName;
use std::convert::TryInto;
use tide::http::{self, url::Url, Method};
use tide::Middleware;

use test_utils::BoxFuture;

mod test_utils;

#[derive(Debug)]
struct TestMiddleware(HeaderName, &'static str);

impl TestMiddleware {
    fn with_header_name(name: &'static str, value: &'static str) -> Self {
        Self(name.try_into().unwrap(), value)
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for TestMiddleware {
    fn handle<'a>(
        &'a self,
        req: tide::Request<State>,
        next: tide::Next<'a, State>,
    ) -> BoxFuture<'a, tide::Result<tide::Response>> {
        Box::pin(async move {
            let mut res = next.run(req).await;
            res.insert_header(self.0.clone(), self.1);
            Ok(res)
        })
    }
}

async fn echo_path<State>(req: tide::Request<State>) -> tide::Result<String> {
    Ok(req.url().path().to_string())
}

#[async_std::test]
async fn route_middleware() {
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

    let req = http::Request::new(Method::Get, Url::parse("http://localhost/foo").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res["X-Foo"], "foo");

    let req = http::Request::new(Method::Post, Url::parse("http://localhost/foo").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res["X-Foo"], "foo");

    let req = http::Request::new(Method::Put, Url::parse("http://localhost/foo").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert!(res.header("X-Foo").is_none());

    let req = http::Request::new(Method::Get, Url::parse("http://localhost/foo/bar").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res["X-Foo"], "foo");
    assert_eq!(res["x-bar"], "bar");
}

#[async_std::test]
async fn app_and_route_middleware() {
    let mut app = tide::new();
    app.middleware(TestMiddleware::with_header_name("X-Root", "root"));
    app.at("/foo")
        .middleware(TestMiddleware::with_header_name("X-Foo", "foo"))
        .get(echo_path);
    app.at("/bar")
        .middleware(TestMiddleware::with_header_name("X-Bar", "bar"))
        .get(echo_path);

    let req = http::Request::new(Method::Get, Url::parse("http://localhost/foo").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res["X-Root"], "root");
    assert_eq!(res["x-foo"], "foo");

    assert!(res.header("x-bar").is_none());

    let req = http::Request::new(Method::Get, Url::parse("http://localhost/bar").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res["X-Root"], "root");
    assert!(res.header("x-foo").is_none());
    assert_eq!(res["X-Bar"], "bar");
}

#[async_std::test]
async fn nested_app_with_route_middleware() {
    let mut inner = tide::new();
    inner.middleware(TestMiddleware::with_header_name("X-Inner", "inner"));
    inner
        .at("/baz")
        .middleware(TestMiddleware::with_header_name("X-Baz", "baz"))
        .get(echo_path);

    let mut app = tide::new();
    app.middleware(TestMiddleware::with_header_name("X-Root", "root"));
    app.at("/foo")
        .middleware(TestMiddleware::with_header_name("X-Foo", "foo"))
        .get(echo_path);
    app.at("/bar")
        .middleware(TestMiddleware::with_header_name("X-Bar", "bar"))
        .nest(inner);

    let req = http::Request::new(Method::Get, Url::parse("http://localhost/foo").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res["X-Root"], "root");
    assert!(res.header("X-Inner").is_none());
    assert_eq!(res["X-Foo"], "foo");
    assert!(res.header("X-Bar").is_none());
    assert!(res.header("X-Baz").is_none());

    let req = http::Request::new(Method::Get, Url::parse("http://localhost/bar/baz").unwrap());
    let res: http::Response = app.respond(req).await.unwrap();
    assert_eq!(res["X-Root"], "root");
    assert_eq!(res["X-Inner"], "inner");
    assert!(res.header("X-Foo").is_none());
    assert_eq!(res["X-Bar"], "bar");
    assert_eq!(res["X-Baz"], "baz");
}

#[async_std::test]
async fn subroute_not_nested() {
    let mut app = tide::new();
    app.at("/parent") // /parent
        .middleware(TestMiddleware::with_header_name("X-Parent", "Parent"))
        .get(echo_path);
    app.at("/parent/child") // /parent/child, not nested
        .middleware(TestMiddleware::with_header_name("X-Child", "child"))
        .get(echo_path);

    let req = http::Request::new(
        Method::Get,
        Url::parse("http://localhost/parent/child").unwrap(),
    );
    let res: http::Response = app.respond(req).await.unwrap();
    assert!(res.header("X-Parent").is_none());
    assert_eq!(res["x-child"], "child");
}
