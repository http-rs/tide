use tide::Request;

async fn echo_path(cx: Request<()>) -> String {
    let path: String = cx.param("path").unwrap();
    format!("Your path is: {}", path)
}

fn main() {
    let mut app = tide::Server::new();
    app.at("/echo_path/*path").get(echo_path);
    app.run("127.0.0.1:8000").unwrap();
}
