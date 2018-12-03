// A `Router`-nesting version of the example `named_path`.

#![feature(async_await, futures_api)]

use tide::{
    head::{Named, NamedSegment},
    Router,
};

struct Number(i32);

impl NamedSegment for Number {
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

fn build_add_two<Data: Clone + Send + Sync + 'static>(router: &mut Router<Data>) {
    router.at("{num}").get(add_two);
}

fn main() {
    let mut app = tide::App::new(());
    app.path("add_two").nest(build_add_two);

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
