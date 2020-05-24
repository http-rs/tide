fn middleware<'a, State: Send + Sync + 'static>(
    request: tide::Request<State>,
    next: tide::Next<'a, State>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = tide::Result> + Send + 'a>> {
    Box::pin(async {
        tide::log::info!("before");
        let result = next.run(request).await;
        tide::log::info!("after");
        result
    })
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    let mut app = tide::new();

    app.middleware(middleware);

    app.at("/").get(|_| async move { Ok("Hello, world!") });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
