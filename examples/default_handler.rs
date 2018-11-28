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

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address)
}
