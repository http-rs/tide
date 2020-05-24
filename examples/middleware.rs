async fn middleware<'a, State: 'static>(
    request: tide::Request<State>,
    next: tide::Next<'a, State>,
) -> tide::Result {
    tide::log::info!("middleware before");
    let result = next.run(request).await;
    tide::log::info!("middleware after");
    result
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();
    app.middleware(middleware);
    app.at("/").get(|_| async move { Ok("Hello, world!") });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
