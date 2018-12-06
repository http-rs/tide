#![feature(async_await, futures_api)]
extern crate basic_cookies;

use basic_cookies::Cookie;
use std::collections::HashMap;
use tide::{Compute, Computed, CookieBuilder, Request};

#[derive(Clone, Debug)]
struct Cookies(HashMap<String, tide::Cookie<String>>);

impl Compute for Cookies {
    fn compute_fresh(req: &mut Request) -> Self {
        let unique_cookies = if let Some(raw_cookies) = req.headers().get("Cookie") {
            Cookie::parse(raw_cookies.to_str().unwrap())
                .unwrap()
                .iter()
                .map(|c| {
                    let cookie = CookieBuilder::new()
                        .name(c.get_name().into())
                        .value(c.get_value().into())
                        .build();
                    (c.get_name().into(), cookie).into()
                })
                .collect()
        } else {
            HashMap::new()
        };
        Cookies(unique_cookies)
    }
}

async fn hello_cookies(Computed(cookies): Computed<Cookies>) -> String {
    format!("hello cookies: {:?}", cookies)
}

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get(hello_cookies);

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
