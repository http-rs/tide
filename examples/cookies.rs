#![feature(async_await)]

use cookie::Cookie;
use tide::{cookies::CookiesExt, middleware::CookiesMiddleware, Context};

/// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
///
async fn retrieve_cookie(mut cx: Context<()>) -> String {
    format!("hello cookies: {:?}", cx.get_cookie("hello").unwrap())
}

#[allow(unused_mut)] // Workaround clippy bug
async fn set_cookie(mut cx: Context<()>) {
    cx.set_cookie(Cookie::new("hello", "world")).unwrap();
}

#[allow(unused_mut)] // Workaround clippy bug
async fn remove_cookie(mut cx: Context<()>) {
    cx.remove_cookie(Cookie::named("hello")).unwrap();
}

fn main() {
    let mut app = tide::App::new();
    app.middleware(CookiesMiddleware::new());

    app.at("/").get(retrieve_cookie);
    app.at("/set").get(set_cookie);
    app.at("/remove").get(remove_cookie);
    app.serve("127.0.0.1:8000").unwrap();
}
