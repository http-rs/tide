#![feature(async_await)]

#[runtime::main]
async fn main() -> Result<(), tide::Exception> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut app = tide::new();
    app.middleware(tide::middleware::RequestLogger::new());
    app.at("/").get(|_| async move { "Hello, world!" });
    app.bind("127.0.0.1:8000").await?;
    Ok(())
}
