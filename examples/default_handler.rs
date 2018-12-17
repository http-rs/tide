#![feature(async_await)]

use http::status::StatusCode;
use tide::body;

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get(async || "Hello, world!");

    app.default_handler(async || {
        http::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body(body::Body::from("¯\\_(ツ)_/¯".to_string().into_bytes()))
            .unwrap()
    });

    app.serve()
}
