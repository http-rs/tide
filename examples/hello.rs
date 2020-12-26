use tide::limit::LoadShedder;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    tide::log::start();
    let mut app = tide::new();
    app.with(LoadShedder::new(0));
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
