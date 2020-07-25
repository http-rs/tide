#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::default();
    app.at("/").get(|_| async { Ok("visit /src/*") });
    app.at("/src").serve_dir("src/")?;
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
