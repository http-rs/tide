#![feature(async_await)]

use tide::middleware::CorsMiddleware;
use http::header::HeaderValue;

fn main() {
    let mut app = tide::App::new();

    app.middleware(
        CorsMiddleware::new()
        .allow_origin(HeaderValue::from_static("*"))
        .allow_methods(HeaderValue::from_static("GET, POST, OPTION"))
        .echo_back_origin(true)
    );

    app.at("/").get(async move |_| "Hello, world!");

    app.run("127.0.0.1:8000").unwrap();
}
