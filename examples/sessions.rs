#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    let mut app = tide::new();

    let db = "tide-sessions";
    let coll_name = "sessions";
    let store = async_mongodb_session::MongodbSessionStore::connect(
        "mongodb://localhost:27017",
        db,
        coll_name,
    )
    .await?;
    app.middleware(tide::sessions::SessionMiddleware::new(
        store,
        std::env::var("TIDE_SECRET")
            .unwrap_or(
                "Please provide a TIDE_SECRET value of at \
                      least 32 bytes in order to run this example"
                    .to_owned(),
            )
            .as_bytes(),
    ));

    app.middleware(tide::utils::Before(
        |mut request: tide::Request<()>| async move {
            let session = request.session_mut();
            let visits: usize = session.get("visits").unwrap_or_default();
            session.insert("visits", visits + 1).unwrap();
            request
        },
    ));

    app.at("/").get(|req: tide::Request<()>| async move {
        let visits: usize = req.session().get("visits").unwrap();
        Ok(format!("you have visited this website {} times", visits))
    });

    app.at("/reset")
        .get(|mut req: tide::Request<()>| async move {
            req.session_mut().destroy();
            Ok(tide::Redirect::new("/"))
        });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
