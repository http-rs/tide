use async_std::io::prelude::*;
use futures::executor::block_on;
use http_service::Body;
use http_service_mock::make_server;

#[test]
fn nested() {
    let mut inner = tide::new();
    // Removing prefixes isn't implemented yet, so write prefixes everywhere
    inner.at("/foo/foo").get(|_| async { "foo" });
    inner.at("/foo/bar").get(|_| async { "bar" });

    let mut outer = tide::new();
    // Nest the inner app on /foo
    outer.at("/foo/*").get(inner.into_http_service());

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
