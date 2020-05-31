use tide::{log, Request, Response};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    log::start();

    let mut app = tide::new();
    app.at("/").get(serve_html);
    app.at("/src").serve_dir("target/")?;
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn serve_html(_: Request<()>) -> tide::Result {
    let html = html_index::new()
        .title("Tide live-reloading WASM server example");
    Ok(Response::from_res(html))
}
