use async_mongodb_session::MongodbSessionStore;
use tide::sessions::SessionMiddleware;
use tide::utils::Before;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    let db = "tide-sessions";
    let coll_name = "sessions";
    let store = MongodbSessionStore::connect("mongodb://localhost:27017", db, coll_name).await?;

    let secret = std::env::var("TIDE_SECRET").unwrap();
    app.middleware(SessionMiddleware::new(store, secret.as_bytes()));

    app.middleware(Before(|mut req: tide::Request<()>| async move {
        let session = req.session_mut();
        let visits: usize = session.get("visits").unwrap_or_default();
        session.insert("visits", visits + 1).unwrap();
        req
    }));

    app.at("/").get(|req: tide::Request<()>| async move {
        let visits: usize = req.session().get("visits").unwrap();
        Ok(format!("you have visited this website {} times", visits))
    });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
