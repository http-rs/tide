use tide::{http, Result, Server};

use std::fs::{self, File};
use std::io::Write;

fn api() -> Box<dyn tide::Endpoint> {
    Box::new(|_| async { Ok("api") })
}

fn app(tempdir: &tempfile::TempDir) -> Result<Server> {
    let static_dir = tempdir.path().join("static");
    fs::create_dir(&static_dir)?;

    let file_path = static_dir.join("foo");
    let mut file = File::create(&file_path)?;
    write!(file, "Foobar")?;

    let mut app = Server::new();
    app.at("/static/api").get(api());
    app.at("/static/*")
        .serve_dir(static_dir.to_str().unwrap())?;

    Ok(app)
}

fn request(path: &str) -> http_types::Request {
    http_types::Request::get(
        http_types::Url::parse(&format!("http://localhost/static/{}", path)).unwrap(),
    )
}

#[async_std::test]
async fn ok() {
    let tempdir = tempfile::tempdir().unwrap();
    let app = app(&tempdir).unwrap();
    let mut res: http::Response = app.respond(request("foo")).await.unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap().as_str(), "Foobar");
}

#[async_std::test]
async fn not_found() {
    let tempdir = tempfile::tempdir().unwrap();
    let app = app(&tempdir).unwrap();
    let res: http::Response = app.respond(request("bar")).await.unwrap();

    assert_eq!(res.status(), 404);
}

#[async_std::test]
async fn static_endpoint_mixin() {
    let tempdir = tempfile::tempdir().unwrap();
    let app = app(&tempdir).unwrap();
    let mut res: http::Response = app.respond(request("api")).await.unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(res.body_string().await.unwrap().as_str(), "api");
}
