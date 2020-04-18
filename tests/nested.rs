use futures::future::BoxFuture;
use http_service_mock::make_server;
use http_types::headers::{HeaderName, HeaderValue};
use http_types::{Method, Request, Url};
use std::str::FromStr;

#[async_std::test]
async fn nested() {
    let mut inner = tide::new();
    inner.at("/foo").get(|_| async { Ok("foo") });
    inner.at("/bar").get(|_| async { Ok("bar") });

    let mut outer = tide::new();
    // Nest the inner app on /foo
    outer.at("/foo").nest(inner);

    let mut server = make_server(outer.into_http_service()).unwrap();

    let req = Request::new(
        Method::Get,
        Url::parse("http://example.com/foo/foo").unwrap(),
    );
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "foo");

    let req = Request::new(
        Method::Get,
        Url::parse("http://example.com/foo/bar").unwrap(),
    );
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "bar");
}

#[async_std::test]
async fn nested_middleware() {
    let echo_path = |req: tide::Request<()>| async move { Ok(req.uri().path().to_string()) };
    fn test_middleware(
        req: tide::Request<()>,
        next: tide::Next<'_, ()>,
    ) -> BoxFuture<'_, tide::Result<tide::Response>> {
        Box::pin(async move {
            let res = next.run(req).await?;
            let res = res.set_header(
                HeaderName::from_ascii("X-Tide-Test".to_owned().into_bytes()).unwrap(),
                "1",
            );
            Ok(res)
        })
    }

    let mut app = tide::new();

    let mut inner_app = tide::new();
    inner_app.middleware(test_middleware);
    inner_app.at("/echo").get(echo_path);
    inner_app.at("/:foo/bar").strip_prefix().get(echo_path);
    app.at("/foo").nest(inner_app);

    app.at("/bar").get(echo_path);

    let mut server = make_server(app.into_http_service()).unwrap();

    let req = Request::new(
        Method::Get,
        Url::parse("http://example.com/foo/echo").unwrap(),
    );
    let res = server.simulate(req).unwrap();
    assert_header(&res, "X-Tide-Test", Some("1"));
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "/echo");

    let req = Request::new(
        Method::Get,
        Url::parse("http://example.com/foo/x/bar").unwrap(),
    );
    let res = server.simulate(req).unwrap();
    assert_header(&res, "X-Tide-Test", Some("1"));
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "/");

    let req = Request::new(Method::Get, Url::parse("http://example.com/bar").unwrap());
    let res = server.simulate(req).unwrap();
    assert_header(&res, "X-Tide-Test", None);
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
    outer.at("/").get(|_| async move { Ok("Hello, world!") });
    outer.at("/foo").nest(inner);

    let mut server = make_server(outer.into_http_service()).unwrap();

    let req = Request::new(Method::Get, Url::parse("http://example.com/foo").unwrap());
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "the number is 42");

    let req = Request::new(Method::Get, Url::parse("http://example.com/").unwrap());
    let res = server.simulate(req).unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap(), "Hello, world!");
}

// See https://github.com/http-rs/http-types/issues/89 for a proposed fix to this boilerplate.
fn assert_header(headers: impl AsRef<http_types::Headers>, lhs: &str, rhs: Option<&str>) {
    match rhs {
        Some(s) => {
            let header = headers
                .as_ref()
                .get(
                    &http_types::headers::HeaderName::from_ascii(lhs.to_owned().into_bytes())
                        .unwrap(),
                )
                .unwrap()
                .iter()
                .next();
            assert_eq!(header, Some(&HeaderValue::from_str(s).unwrap()));
        }
        None => {
            let header = headers.as_ref().get(
                &http_types::headers::HeaderName::from_ascii(lhs.to_owned().into_bytes()).unwrap(),
            );
            assert_eq!(header, None);
        }
    }
}
