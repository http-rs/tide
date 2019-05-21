//! Real World example application.
//!
//! https://github.com/gothinkster/realworld/tree/master/api

#![feature(async_await)]

async fn noop(_cx: tide::Context<()>) -> String {
    "{}".to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut app = tide::App::new();

    app.at("/user").get(noop);
    app.at("/user").put(noop);
    app.at("/users").nest(|app| {
        app.at("/").post(noop);
        app.at("/login").post(noop);
    });

    app.at("/articles").nest(|app| {
        app.at("/").get(noop);
        app.at("/").post(noop);
        app.at("/feed").get(noop);
        app.at("/:slug").get(noop);
        app.at("/:slug").put(noop);
        app.at("/:slug").delete(noop);
        app.at("/:slug/favorite").put(noop);
        app.at("/:slug/favorite").delete(noop);
        app.at("/:slug/comments").get(noop);
        app.at("/:slug/comments").post(noop);
        app.at("/:slug/comments/:id").delete(noop);
    });
    app.run("localhost:8080")?;
    Ok(())
}
