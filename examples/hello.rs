#![feature(async_await)]
fn main() {
    let mut app = tide::new();
    app.at("/").get(|_| async move { "Hello, world!" });
    app.bind("127.0.0.1:8000").unwrap();
}
