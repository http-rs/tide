use async_std::task;

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/").get(|_| async move { Ok("Hello, world!") });
        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
