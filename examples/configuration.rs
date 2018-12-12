#![feature(async_await, futures_api)]

use tide::{
    head::Path,
    ExtractConfiguration,
};

#[derive(Clone, Default)]
struct IncreaseBy(i32);

async fn add(
    Path(base): Path<i32>,
    ExtractConfiguration(amount): ExtractConfiguration<IncreaseBy>,
) -> String {
    let IncreaseBy(amount) = amount.unwrap_or_default();
    format!("{} plus {} is {}", base, amount, base + amount)
}

fn main() {
    let mut app = tide::App::new(());
    app.config(IncreaseBy(1));
    app.at("add_one/{}").get(add);
    app.at("add_two/{}").get(add).config(IncreaseBy(2));

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
