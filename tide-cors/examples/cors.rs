#![feature(async_await)]

use http::header::HeaderValue;
use tide::middleware::{CorsMiddleware, CorsOrigin};

fn main() {
    let mut app = tide::App::new();

    app.middleware(
        CorsMiddleware::new()
            .allow_origin(CorsOrigin::from("*"))
            .allow_methods(HeaderValue::from_static("GET, POST, OPTIONS")),
    );

    app.at("/").get(async move |_| "Hello, world!");

    app.run("127.0.0.1:8000").unwrap();
}
