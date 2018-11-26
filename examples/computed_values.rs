#![feature(async_await, futures_api)]
extern crate cookie;

use cookie::Cookie;
use tide::{Compute, Computed, Request};

#[derive(Clone, Debug)]
struct Cookies {
    content: Option<CookieCream>,
}

#[derive(Clone, Debug)]
struct CookieCream {
    name: String,
    value: String,
}

impl Compute for Cookies {
    fn compute_fresh(req: &mut Request) -> Self {
        let content = if let Some(raw_cookies) = req.headers().get("Cookie") {
            let cookie = Cookie::parse(raw_cookies.to_str().unwrap()).unwrap();
            Some(CookieCream {
                name: cookie.name().into(),
                value: cookie.value().into(),
            })
        } else {
            None
        };

        Cookies { content }
    }
}

async fn hello_cookies(cookies: Computed<Cookies>) -> String {
    let Computed(cookies) = cookies;
    format!("hello cookies: {:?}", cookies)
}

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get(hello_cookies);
    app.serve("127.0.0.1:7878")
}
