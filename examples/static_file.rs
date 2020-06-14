#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();
    app.at("/").get(|_| async { "visit /src/*" });
    app.at("/src").serve_dir("src/")?;
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
