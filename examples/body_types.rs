use serde::{Deserialize, Serialize};
use tide::{
    error::ResultExt,
    forms::{self, ContextExt},
    response, App, Context, EndpointResult,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    author: Option<String>,
    contents: String,
}

async fn echo_string(mut cx: Context<()>) -> String {
    let msg = cx.body_string().await.unwrap();
    println!("String: {}", msg);
    msg
}

async fn echo_bytes(mut cx: Context<()>) -> Vec<u8> {
    let msg = cx.body_bytes().await.unwrap();
    println!("Bytes: {:?}", msg);
    msg
}

async fn echo_json(mut cx: Context<()>) -> EndpointResult {
    let msg: Message = cx.body_json().await.client_err()?;
    println!("JSON: {:?}", msg);
    Ok(response::json(msg))
}

async fn echo_form(mut cx: Context<()>) -> EndpointResult {
    let msg: Message = cx.body_form().await?;
    println!("Form: {:?}", msg);
    Ok(forms::form(msg))
}

fn main() {
    let mut app = App::new();

    app.at("/echo/string").post(echo_string);
    app.at("/echo/bytes").post(echo_bytes);
    app.at("/echo/json").post(echo_json);
    app.at("/echo/form").post(echo_form);

    app.run("127.0.0.1:8000").unwrap();
}
