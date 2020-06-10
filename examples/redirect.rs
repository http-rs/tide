use tide::{http::StatusCode, Redirect, Response};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.at("/").get(|_| async { Ok("Root") });

    // Redirect hackers to YouTube.
    app.at("/.env")
        .get(Redirect::new("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));

    app.at("/users-page").get(|_| async {
        Ok(if signed_in() {
            Response::new(StatusCode::Ok)
        } else {
            // If the user is not signed in then lets redirect them to home page.
            Redirect::new("/").into()
        })
    });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn signed_in() -> bool {
    false
}
