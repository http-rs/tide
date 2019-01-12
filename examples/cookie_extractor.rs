#![feature(async_await, futures_api)]

use tide::Cookies;

/// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
///
async fn hello_cookies(cookies: Cookies) -> String {
    format!("hello cookies: {:?}", cookies)
}

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get(hello_cookies);

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
