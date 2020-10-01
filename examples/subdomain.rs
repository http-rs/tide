#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();
    app.subdomain(":user")
        .at("/")
        .get(|req: tide::Request<()>| async move {
            let user = req.param::<String>("user").unwrap();
            Ok(format!("Welcome user {}", user))
        });
    app.at("/")
        .get(|_| async { Ok("Welcome to my landing page") });
    app.subdomain("blog")
        .at("/")
        .get(|_| async { Ok("Welcome to my blog") });

    // to be able to use this example, please note some domains down inside of
    // your /etc/hosts file. Add the following:
    // 127.0.0.1 example.local
    // 127.0.0.1 blog.example.local
    // 127.0.0.1 tom.example.local

    // After add the following urls. Test it inside of your browser.
    app.listen("http://example.local:8080").await?;
    Ok(())
}
