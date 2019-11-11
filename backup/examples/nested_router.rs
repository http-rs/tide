use tide::Server;

fn main() {
    let mut app = Server::new();
    app.at("/gates").nest(|router| {
        router
            .at("/")
            .get(|_| async move { "This is an area in front of the gates" });
        router.at("/open").get(|_| async move { "Open the gates!" });
        router
            .at("/close")
            .get(|_| async move { "Close the gates!" });
    });
    app.run("127.0.0.1:8000").unwrap();
}
