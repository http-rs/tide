#![feature(async_await, futures_api)]

#[macro_use]
extern crate serde_derive;
use tide::body;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    author: Option<String>,
    contents: String,
}

async fn echo_string(msg: body::Str) -> String {
    println!("String: {}", *msg);
    format!("{}", *msg)
}

async fn echo_string_lossy(msg: body::StrLossy) -> String {
    println!("String: {}", *msg);
    format!("{}", *msg)
}

async fn echo_vec(msg: body::Bytes) -> String {
    println!("Vec<u8>: {:?}", *msg);

    String::from_utf8(msg.to_vec()).unwrap()
}

async fn echo_json(msg: body::Json<Message>) -> body::Json<Message> {
    println!("JSON: {:?}", *msg);

    msg
}

async fn echo_form(msg: body::Form<Message>) -> body::Form<Message> {
    println!("Form: {:?}", *msg);

    msg
}

fn main() {
    let mut app = tide::App::new(());

    app.at("/echo/string").post(echo_string);
    app.at("/echo/string_lossy").post(echo_string_lossy);
    app.at("/echo/vec").post(echo_vec);
    app.at("/echo/json").post(echo_json);
    app.at("/echo/form").post(echo_form);

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
