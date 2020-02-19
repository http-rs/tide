use http_types::{self, Method, Url};
use tide::{self, Request, Response, Result, StatusCode};

#[async_std::test]
async fn test_param_invalid_type() {
    async fn get_by_id(req: Request<()>) -> Result<Response> {
        assert_eq!(
            req.param::<i32>("id").unwrap_err().to_string(),
            "Param failed to parse: invalid digit found in string"
        );
        let _ = req.param::<i32>("id")?;
        Result::Ok(Response::new(StatusCode::Ok))
    }
    let mut server = tide::new();
    server.at("/by_id/:id").get(get_by_id);

    let req = http_types::Request::new(
        Method::Get,
        Url::parse("http://example.com/by_id/wrong").unwrap(),
    );
    let res: http_types::Response = server.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::InternalServerError);
}

#[async_std::test]
async fn test_missing_param() {
    async fn greet(req: Request<()>) -> Result<Response> {
        assert_eq!(
            req.param::<String>("name").unwrap_err().to_string(),
            "Param \"name\" not found!"
        );
        let _: String = req.param("name")?;
        Result::Ok(Response::new(StatusCode::Ok))
    }
    let mut server = tide::new();
    server.at("/").get(greet);

    let req = http_types::Request::new(Method::Get, Url::parse("http://example.com/").unwrap());
    let res: http_types::Response = server.respond(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::InternalServerError);
}

#[async_std::test]
async fn hello_world_parametrized() {
    async fn greet(req: tide::Request<()>) -> Result<Response> {
        let name = req.param("name").unwrap_or("nori".to_owned());
        let mut response = tide::Response::new(StatusCode::Ok);
        response.set_body(format!("{} says hello", name));
        Ok(response)
    }

    let mut server = tide::new();
    server.at("/").get(greet);
    server.at("/:name").get(greet);

    let req = http_types::Request::new(Method::Get, Url::parse("http://example.com/").unwrap());
    let mut res: http_types::Response = server.respond(req).await.unwrap();
    assert_eq!(
        res.body_string().await.unwrap(),
        "nori says hello".to_string()
    );

    let req = http_types::Request::new(Method::Get, Url::parse("http://example.com/iron").unwrap());
    let mut res: http_types::Response = server.respond(req).await.unwrap();
    assert_eq!(
        res.body_string().await.unwrap(),
        "iron says hello".to_string()
    );
}
