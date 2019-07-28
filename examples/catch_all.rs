#![feature(async_await)]
use tide::Context;

async fn echo_path(cx: Context<()>) -> String {
    let path: String = cx.param("path").unwrap();
    format!("Your path is: {}", path)
}

fn main() {
    let mut app = tide::new();
    app.at("/echo_path/*path").get(echo_path);
    app.bind("127.0.0.1:8000").await?;
    Ok(())
}
