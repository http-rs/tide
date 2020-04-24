use async_std::task;

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/").get(|_| async move { "Root" });
        app.at("/api").nest({
            let mut api = tide::new();
            api.at("/hello").get(|_| async move { "Hello, world" });
            api.at("/goodbye").get(|_| async move { "Goodbye, world" });
            api
        });
        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
