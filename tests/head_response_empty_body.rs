use tide::http::{Method, Request, Response, Url};
#[async_std::test]
async fn head_response_empty() {
    let mut app = tide::new();

    app.at("/")
        .get(|_| async { Ok("this shouldn't exist in the body of a HEAD response") });

    let req = Request::new(Method::Head, Url::parse("http://example.com/").unwrap());
    let mut res: Response = app.respond(req).await.unwrap();

    let body = res.body_string().await.unwrap();
    assert!(body.is_empty());
}
