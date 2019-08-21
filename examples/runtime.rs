/// An example of how to run a Tide service on top of `runtime`, this also shows the pieces
/// necessary if you wish to run a service on some other executor/IO source.

#[runtime::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // First, we create a simple hello world application
    let mut app = tide::App::new();
    app.at("/").get(|_| async move { "Hello, world!" });

    // Instead of using `App::run` to start the application, which implicitly uses a default
    // http-service server, we need to configure a custom server with the executor and IO source we
    // want it to use and then run the Tide service on it.

    // Turn the `tide::App` into a generic `http_service::HttpService`
    let http_service = app.into_http_service();

    // Build an `http_service_hyper::Server` using runtime's `TcpListener` and `Spawn` instances
    // instead of hyper's defaults.
    let mut listener = runtime::net::TcpListener::bind("127.0.0.1:8000")?;
    let server = http_service_hyper::Server::builder(listener.incoming())
        .with_spawner(runtime::task::Spawner::new());

    // Serve the Tide service on the configured server, and wait for it to complete
    server.serve(http_service).await?;

    Ok(())
}
