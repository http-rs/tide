#![feature(async_await)]
fn main() {
    use log::LevelFilter;
    use log4rs::append::console::ConsoleServerender;
    use log4rs::config::{Serverender, Config, Root};

    let stdout = ConsoleServerender::builder().build();
    let config = Config::builder()
        .appender(Serverender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let mut app = tide::Server::new();
    app.middleware(tide::middleware::RequestLogger::new());
    app.at("/").get(|_| async move { "Hello, world!" });
    app.run("127.0.0.1:8000").unwrap();
}
