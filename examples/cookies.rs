use async_std::task;
use cookie::Cookie;
use tide::{middleware::CookiesMiddleware, Request, Response};

/// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
///
async fn retrieve_cookie(cx: Request<()>) -> String {
    format!("hello cookies: {:?}", cx.cookie("hello").unwrap())
}

async fn set_cookie(mut cx: Request<()>) -> Response {
    cx.set_cookie(Cookie::new("hello", "world")).unwrap();
    tide::Response::new(200)
}

async fn remove_cookie(mut cx: Request<()>) -> Response {
    cx.remove_cookie(Cookie::named("hello")).unwrap();
    tide::Response::new(200)
}

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.middleware(CookiesMiddleware::new());

        app.at("/").get(retrieve_cookie);
        app.at("/set").get(set_cookie);
        app.at("/remove").get(remove_cookie);
        app.listen("127.0.0.1:8080").await?;

        Ok(())
    })
}
