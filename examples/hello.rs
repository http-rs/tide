fn main() {
    let mut app = tide::App::new();
    app.at("/").get(|_| async move { "Hello, world!" });
    app.serve("127.0.0.1:8000").unwrap();
}
