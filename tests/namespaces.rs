use std::{future::Future, pin::Pin};

use http::Url;
use tide::http::{self, Method};

async fn success(_: tide::Request<()>) -> Result<tide::Response, tide::Error> {
    let mut res = tide::Response::new(200);
    res.set_body("success");
    Ok(res)
}

async fn echo_path(req: tide::Request<()>) -> Result<String, tide::Error> {
    match req.param("path") {
        Ok(path) => Ok(path.to_owned()),
        Err(err) => Err(tide::Error::from_str(tide::StatusCode::BadRequest, err)),
    }
}

async fn multiple_echo_path(req: tide::Request<()>) -> Result<String, tide::Error> {
    let err = |err| tide::Error::from_str(tide::StatusCode::BadRequest, err);
    let path = req.param("path").map_err(err)?;
    let user = req.param("user").map_err(err)?;
    Ok(format!("{} {}", path, user))
}

fn test_middleware<'a>(
    _: tide::Request<()>,
    _: tide::Next<'a, ()>,
) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
    Box::pin(async {
        let mut res = tide::Response::new(200);
        res.set_body("middleware return");
        Ok(res)
    })
}

#[async_std::test]
async fn subdomain() {
    let mut app = tide::Server::new();
    app.subdomain("api").at("/").get(success);
    let url: Url = "http://api.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "success");
}

#[async_std::test]
async fn multiple_subdomain() {
    let mut app = tide::Server::new();
    app.subdomain("this.for.subdomain.length")
        .at("/")
        .get(success);
    let url: Url = "http://this.for.subdomain.length.example.com/"
        .parse()
        .unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "success");
}

#[async_std::test]
async fn subdomain_with_path_params() {
    let mut app = tide::Server::new();
    app.subdomain("api").at("/:path").get(echo_path);
    let url: Url = "http://api.example.com/subdomain-work".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "subdomain-work");
}

#[async_std::test]
async fn multiple_registered_subdomains() {
    let mut app = tide::Server::new();
    app.subdomain("blog").at("/").get(success);
    app.subdomain("api").at("/:path").get(echo_path);

    let url: Url = "http://blog.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "success");

    let url: Url = "http://api.example.com/subdomain-work".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "subdomain-work");
}

#[async_std::test]
async fn subdomain_with_middleware() {
    let mut app = tide::Server::new();
    app.subdomain("api")
        .with(test_middleware)
        .at("/")
        .get(success);

    let url: Url = "http://api.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "middleware return");
}

#[async_std::test]
async fn subdomain_params() {
    let mut app = tide::Server::new();
    app.subdomain(":path").at("/").get(echo_path);
    let url: Url = "http://example.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "example");
}

#[async_std::test]
async fn subdomain_multiple_params() {
    let mut app = tide::Server::new();
    app.subdomain(":path.:user").at("/").get(multiple_echo_path);
    let url: Url = "http://example.tommy.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "example tommy");
}

#[async_std::test]
async fn subdomain_wildcard() {
    let mut app = tide::Server::new();
    app.subdomain("*").at("/").get(success);

    let url: Url = "http://example.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "success");

    let url: Url = "http://user.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "success");

    let url: Url = "http://example.user.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 404);
}

#[async_std::test]
async fn subdomain_routing() {
    let mut app = tide::Server::new();
    // setup
    app.at("/").get(|_| async { Ok("landing page") });
    app.subdomain("blog")
        .at("/")
        .get(|_| async { Ok("my blog") });
    app.subdomain(":user")
        .at("/")
        .get(|req: tide::Request<()>| async move {
            let user = req.param("user").unwrap();
            Ok(format!("user {}", user))
        });

    // testing
    let url: Url = "http://example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "landing page");

    let url: Url = "http://blog.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "my blog");

    let url: Url = "http://tom.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "user tom");

    let url: Url = "http://user.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "user user");
}

#[async_std::test]
async fn subdomain_routing_wildcard() {
    let mut app = tide::Server::new();
    // setup
    app.subdomain(":user")
        .at("/")
        .get(|req: tide::Request<()>| async move {
            let user = req.param("user").unwrap();
            Ok(format!("user {}", user))
        });
    app.at("/").get(|_| async { Ok("landing page") });
    app.subdomain("blog")
        .at("/")
        .get(|_| async { Ok("my blog") });

    // testing
    let url: Url = "http://example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "landing page");

    let url: Url = "http://blog.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "my blog");

    let url: Url = "http://tom.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "user tom");

    let url: Url = "http://user.example.com/".parse().unwrap();
    let request = http::Request::new(Method::Get, url);
    let mut response: http::Response = app.respond(request).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.body_string().await.unwrap(), "user user");
}
