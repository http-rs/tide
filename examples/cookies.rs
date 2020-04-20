use async_std::task;
use cookie::Cookie;
use http_types::StatusCode;
use tide::{Request, Response};

/// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
///
async fn retrieve_cookie(cx: Request<()>) -> tide::Result<String> {
    Ok(format!("hello cookies: {:?}", cx.cookie("hello").unwrap()))
}

async fn set_cookie(_req: Request<()>) -> tide::Result<Response> {
    let mut res = tide::Response::new(StatusCode::Ok);
    res.set_cookie(Cookie::new("hello", "world"));
    Ok(res)
}

async fn remove_cookie(_req: Request<()>) -> tide::Result<Response> {
    let mut res = tide::Response::new(StatusCode::Ok);
    res.remove_cookie(Cookie::named("hello"));
    Ok(res)
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
