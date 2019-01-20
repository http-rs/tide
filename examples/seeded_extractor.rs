#![feature(async_await, futures_api)]

use tide::head::{NamedHeader, Header};
use http::header::{HeaderName, HeaderValue};

async fn display_header(value: Header<HeaderValue>) -> String {
    String::from_utf8_lossy(value.0.as_ref()).into_owned()
}

fn main() {
    let mut app = tide::App::new(());
    app.at("/").get_with(display_header, (NamedHeader(HeaderName::from_static("user-agent")),));
    app.serve();
}
