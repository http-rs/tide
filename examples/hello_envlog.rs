#![feature(async_await, async_closure)]
fn main() {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let mut app = tide::App::new();
    app.middleware(tide::middleware::RequestLogger::new());
    app.at("/").get(async move |_| "Hello, world!");
    app.run("127.0.0.1:8000").unwrap();
}
