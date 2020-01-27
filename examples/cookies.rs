use async_std::task;
use cookie::Cookie;
use tide::{Request, Response};

/// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
///
async fn retrieve_cookie(cx: Request<()>) -> String {
    format!("hello cookies: {:?}", cx.cookie("hello").unwrap())
}

async fn set_cookie(_req: Request<()>) -> Response {
    let mut res = tide::Response::new(200);
    res.set_cookie(Cookie::new("hello", "world"));
    res
}

async fn remove_cookie(_req: Request<()>) -> Response {
    let mut res = tide::Response::new(200);
    res.remove_cookie(Cookie::named("hello"));
    res
}

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();

        app.at("/").get(retrieve_cookie);
        app.at("/set").get(set_cookie);
        app.at("/remove").get(remove_cookie);
        app.listen("127.0.0.1:8080").await?;

        Ok(())
    })
}
