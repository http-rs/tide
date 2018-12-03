#![feature(async_await, futures_api)]

async fn echo_path(path: tide::head::Path<String>) -> String {
    format!("Your path is: {}", *path)
}

fn main() {
    let mut app = tide::App::new(());
    app.at("/echo_path").nest(|router| {
        router.at("*").get(echo_path);
    });

    let address = "127.0.0.1:8000".to_owned();
    println!("Server is listening on http://{}", address);
    app.serve(address);
}
