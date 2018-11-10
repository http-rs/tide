#![feature(async_await, futures_api)]

use tide::head::{NamedComponent, Named};

struct Number(i32);

impl NamedComponent for Number {
    const NAME: &'static str = "num";
}

impl std::str::FromStr for Number {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(|num| Number(num))
    }
}

async fn add_two(Named(number): Named<Number>) -> String {
    let Number(num) = number;
    format!("{} plus two is {}", num, num + 2)
}

fn main() {
    let mut app = tide::App::new(());
    app.at("add_two/{num}").get(add_two);
    app.serve("127.0.0.1:8000");
}
