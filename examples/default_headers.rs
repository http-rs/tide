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

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
