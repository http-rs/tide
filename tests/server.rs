use futures::executor::block_on;
use std::sync::Arc;

use tide::*;
use tide::{middleware::Next, router::Selection, Request, Response};

fn simulate_request<'a, State: Default + Clone + Send + Sync + 'static>(
    app: &'a Server<State>,
    path: &'a str,
    method: http::Method,
) -> BoxFuture<'a, Response> {
    let Selection { endpoint, params } = app.router.route(path, method.clone());

    let state = Arc::new(State::default());
    let req = http::Request::builder()
        .method(method)
        .body(http_service::Body::empty())
        .unwrap();
    let cx = Request::new(state, req, params);
    let next = Next {
        endpoint,
        next_middleware: &app.middleware,
    };

    next.run(cx)
}

#[test]
fn simple_static() {
    let mut router = Server::new();
    router.at("/").get(|_| async move { "/" });
    router.at("/foo").get(|_| async move { "/foo" });
    router.at("/foo/bar").get(|_| async move { "/foo/bar" });

    for path in &["/", "/foo", "/foo/bar"] {
        let res = block_on(simulate_request(&router, path, http::Method::GET));
        let body = block_on(res.into_body().into_vec()).expect("Reading body should succeed");
        assert_eq!(&*body, path.as_bytes());
    }
}

#[test]
fn nested_static() {
    let mut router = Server::new();
    router.at("/a").get(|_| async move { "/a" });
    router.at("/b").nest(|router| {
        router.at("/").get(|_| async move { "/b" });
        router.at("/a").get(|_| async move { "/b/a" });
        router.at("/b").get(|_| async move { "/b/b" });
        router.at("/c").nest(|router| {
            router.at("/a").get(|_| async move { "/b/c/a" });
            router.at("/b").get(|_| async move { "/b/c/b" });
        });
        router.at("/d").get(|_| async move { "/b/d" });
    });
    router.at("/a/a").nest(|router| {
        router.at("/a").get(|_| async move { "/a/a/a" });
        router.at("/b").get(|_| async move { "/a/a/b" });
    });
    router.at("/a/b").nest(|router| {
        router.at("/").get(|_| async move { "/a/b" });
    });

    for failing_path in &["/", "/a/a", "/a/b/a"] {
        let res = block_on(simulate_request(&router, failing_path, http::Method::GET));
        if !res.status().is_client_error() {
            panic!(
                "Should have returned a client error when router cannot match with path {}",
                failing_path
            );
        }
    }

    for path in &[
        "/a", "/a/a/a", "/a/a/b", "/a/b", "/b", "/b/a", "/b/b", "/b/c/a", "/b/c/b", "/b/d",
    ] {
        let res = block_on(simulate_request(&router, path, http::Method::GET));
        let body = block_on(res.into_body().into_vec()).expect("Reading body should succeed");
        assert_eq!(&*body, path.as_bytes());
    }
}

#[test]
fn multiple_methods() {
    let mut router = Server::new();
    router.at("/a").nest(|router| {
        router.at("/b").get(|_| async move { "/a/b GET" });
    });
    router.at("/a/b").post(|_| async move { "/a/b POST" });

    for (path, method) in &[("/a/b", http::Method::GET), ("/a/b", http::Method::POST)] {
        let res = block_on(simulate_request(&router, path, method.clone()));
        assert!(res.status().is_success());
        let body = block_on(res.into_body().into_vec()).expect("Reading body should succeed");
        assert_eq!(&*body, format!("{} {}", path, method).as_bytes());
    }
}
