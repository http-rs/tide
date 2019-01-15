#![feature(async_await)]

use http::status::StatusCode;
use tide::IntoResponse;

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get(async || "Hello, world!");

    app.default_handler(async || "¯\\_(ツ)_/¯".with_status(StatusCode::NOT_FOUND));

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address)
}
