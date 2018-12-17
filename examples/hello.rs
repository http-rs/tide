#![feature(async_await)]

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get(async || "Hello, world!");

    app.serve();
}
