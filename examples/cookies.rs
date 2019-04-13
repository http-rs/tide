#![feature(async_await, futures_api)]

use tide::{cookies::ExtractCookies, Context};

/// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
///
async fn hello_cookies(mut cx: Context<()>) -> String {
    format!("hello cookies: {:?}", cx.cookie("hello"))
}

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get(hello_cookies);
    app.serve("127.0.0.1:8000").unwrap();
}
