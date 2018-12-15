#![feature(async_await, futures_api)]

use tide::{head::Path, ExtractConfiguration};

/// A type that represents how much value will be added by the `add` handler.
#[derive(Clone, Default)]
struct IncreaseBy(i32);

async fn add(
    Path(base): Path<i32>,
    // `ExtractConfiguration` will extract the configuration item of given type, and provide it as
    // `Option<T>`. If it is not set, the inner value will be `None`.
    ExtractConfiguration(amount): ExtractConfiguration<IncreaseBy>,
) -> String {
    let IncreaseBy(amount) = amount.unwrap_or_default();
    format!("{} plus {} is {}", base, amount, base + amount)
}

fn main() {
    let mut app = tide::App::new(());
    // `App::config` sets the default configuration of the app (that is, a top-level router).
    app.config(IncreaseBy(1));
    app.at("add_one/{}").get(add); // `IncreaseBy` is set to 1
    app.at("add_two/{}").get(add).config(IncreaseBy(2)); // `IncreaseBy` is overridden to 2

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
