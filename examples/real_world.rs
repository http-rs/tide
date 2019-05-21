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
        app.at("/:slug").nest(|app| {
            app.at("/").get(noop);
            app.at("/").put(noop);
            app.at("/").delete(noop);
            app.at("/favorite").put(noop);
            app.at("/favorite").delete(noop);
            app.at("/comments").get(noop);
            app.at("/comments").post(noop);
            app.at("/comments/:id").delete(noop);
        });
    });

    app.at("/profiles").nest(|app| {
        app.at("/:username").get(noop);
        app.at("/:username/follow").post(noop);
        app.at("/:username/follow").delete(noop);
    });

    app.at("/tags").get(noop);

    app.run("localhost:8080")?;
    Ok(())
}
