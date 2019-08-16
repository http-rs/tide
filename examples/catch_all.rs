use tide::Context;

async fn echo_path(cx: Context<()>) -> String {
    let path: String = cx.param("path").unwrap();
    format!("Your path is: {}", path)
}

#[tokio::main]
async fn main() {
    let mut app = tide::App::new();
    app.at("/echo_path/*path").get(echo_path);
    app.serve("127.0.0.1:8000").await.unwrap();
}
