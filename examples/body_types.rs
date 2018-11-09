#![feature(async_await, futures_api)]

#[macro_use]
extern crate serde_derive;
use tide::body;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    author: Option<String>,
    contents: String,
}

async fn echo_string(msg: String) -> String {
    println!("String: {}", msg);
    format!("{}", msg)
}

async fn echo_vec(msg: Vec<u8>) -> String {
    println!("Vec<u8>: {:?}", msg);

    String::from_utf8(msg).unwrap()
}

async fn echo_json(msg: body::Json<Message>) -> body::Json<Message> {
    println!("JSON: {:?}", msg.0);

    msg
}

fn main() {
    let mut app = tide::App::new(());
    app.at("/echo/string").post(echo_string);
    app.at("/echo/vec").post(echo_vec);
    app.at("/echo/json").post(echo_json);
    app.serve("127.0.0.1:8000");
}
