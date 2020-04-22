use tide::sse;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.at("/sse").get(sse::endpoint(|_req, sender| async move {
        sender.send("fruit", "banana").await;
        sender.send("fruit", "apple").await;
        Ok(())
    }));
    app.listen("localhost:8080").await?;
    Ok(())
}
