#![feature(async_await)]

use tide::{response, App, Context, Response};

async fn hello_world_html(_: Context<()>) -> Response {
    response::html("<!DOCTYPE html><html><body><h1>Hello, World!</h1></body></html>")
}

fn main() {
    let mut app = App::new();
    app.at("/").get(hello_world_html);
    app.run("127.0.0.1:8000").unwrap();
}
