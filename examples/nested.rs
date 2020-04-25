#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.at("/").get(|_| async move { Ok("Root") });
    app.at("/api").nest({
        let mut api = tide::new();
        api.at("/hello").get(|_| async move { Ok("Hello, world") });
        api.at("/goodbye")
            .get(|_| async move { Ok("Goodbye, world") });
        api
    });
    app.listen("127.0.0.1:8080").await
}
