use cookie::Cookie;
use tide::{cookies::RequestExt, middleware::CookiesMiddleware, Request};

/// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
///
async fn retrieve_cookie(cx: Request<()>) -> String {
    format!("hello cookies: {:?}", cx.get_cookie("hello").unwrap())
}

async fn set_cookie(mut cx: Request<()>) {
    cx.set_cookie(Cookie::new("hello", "world")).unwrap();
}

async fn remove_cookie(mut cx: Request<()>) {
    cx.remove_cookie(Cookie::named("hello")).unwrap();
}

fn main() {
    let mut app = tide::Server::new();
    app.middleware(CookiesMiddleware::new());

    app.at("/").get(retrieve_cookie);
    app.at("/set").get(set_cookie);
    app.at("/remove").get(remove_cookie);
    app.run("127.0.0.1:8000").unwrap();
}
