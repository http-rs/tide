#![feature(async_await)]

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get(async || "Hello, world!");

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
