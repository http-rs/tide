use async_std::task;

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/").get(|req: tide::Request<()>| async move {
            Ok(format!(
                "Hello, world! This request was made to: {}",
                req.uri()
            ))
        });
        app.listen_unix("./unix.socket").await?;
        Ok(())
    })
}
