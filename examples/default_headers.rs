#![feature(async_await)]

use tide::middleware::DefaultHeaders;

fn main() {
    let mut app = tide::App::new(());

    app.middleware(
        DefaultHeaders::new()
            .header("X-Version", "1.0.0")
            .header("X-Server", "Tide"),
    );

    app.at("/").get(async || "Hello, world!");

    app.serve();
}
