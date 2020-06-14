use async_std::task;
use tide::Body;

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/")
            .get(|_| async { Body::from_file(file!()).await });
        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
