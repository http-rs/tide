use tide::middleware::DefaultHeaders;

fn main() {
    let mut app = tide::App::new();

    app.middleware(
        DefaultHeaders::new()
            .header("X-Version", "1.0.0")
            .header("X-Server", "Tide"),
    );

    app.at("/").get(async move |_| "Hello, world!");

    app.serve("127.0.0.1:8000").unwrap();
}
