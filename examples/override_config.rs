#![feature(async_await, futures_api)]

use futures::future::FutureObj;
use tide::configuration::{Configuration, Environment, ExtractConfiguration};
use tide::{middleware::RequestContext, Response};

async fn reply(ExtractConfiguration(config): ExtractConfiguration<Configuration>) -> String {
    if let Some(conf) = config {
        println!("We running in the {} environment", conf.env);
        format!("Hello from {}", conf.env)
    } else {
        format!("Unable to read configuration")
    }
}

fn debug_store(ctx: RequestContext<()>) -> FutureObj<Response> {
    println!("{:#?}", ctx.store());
    ctx.next()
}

fn main() {
    let mut app = tide::App::new(());

    // Using the `Configuration` type to build our custon `Configuration` struct we can override
    // the default configuration by using the `.config()` method.
    let updated_conf = Configuration::build()
        .port(8000)
        .env(Environment::Production)
        .finalize();
    app.config(updated_conf);
    app.middleware(debug_store);
    app.at("/").get(reply); // `IncreaseBy` is set to 1

    app.serve();
}
