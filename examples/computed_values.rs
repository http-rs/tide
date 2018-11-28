#![feature(async_await, futures_api)]
extern crate basic_cookies;

use basic_cookies::Cookie;
use std::collections::HashMap;
use tide::{Compute, Computed, Request};

#[derive(Clone, Debug)]
struct Cookies {
    content: HashMap<String, String>,
}

impl Compute for Cookies {
    fn compute_fresh(req: &mut Request) -> Self {
        let content = if let Some(raw_cookies) = req.headers().get("Cookie") {
            Cookie::parse(raw_cookies.to_str().unwrap())
                .unwrap()
                .iter()
                .map(|c| (c.get_name().into(), c.get_value().into()))
                .collect()
        } else {
            HashMap::new()
        };

        Cookies { content }
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
