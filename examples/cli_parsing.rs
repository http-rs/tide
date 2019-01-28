#![feature(async_await, futures_api)]

use structopt::StructOpt;
use tide::{configuration::Configuration, ExtractConfiguration};

#[derive(Debug, StructOpt)]
#[structopt(name = "tide_example")]
struct Settings {
    #[structopt(
        short = "a",
        long = "address",
        env = "TIDE_ADDRESS",
        default_value = "127.0.0.1"
    )]
    address: String,
    #[structopt(short = "p", long = "port", env = "TIDE_PORT", default_value = "8000")]
    port: u16,
    #[structopt(
        short = "d",
        long = "database",
        env = "TIDE_DATABASE",
        default_value = "none"
    )]
    database: String,
}

async fn reply(
    // `ExtractConfiguration` will extract the configuration item of given type, and provide it as
    // `Option<T>`. If it is not set, the inner value will be `None`.
    ExtractConfiguration(config): ExtractConfiguration<Configuration>
) -> String {
    let config = config.unwrap();
    format!("We are running in {:?}", config.env)
}

fn main() {
    let settings = Settings::from_args();
    let app_config = Configuration::build()
        .address(settings.address)
        .port(settings.port)
        .finalize();

    let mut app = tide::App::new(());
    app.config(app_config);

    app.at("/").get(reply);
    app.serve();
}
