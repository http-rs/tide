use tide::{After, Request, Response, Result, StatusCode};

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();
    let mut app = tide::new();

    app.middleware(After(|mut res: Response| async move {
        if let Some(err) = res.downcast_error::<url::ParseError>() {
            let msg = err.to_string().to_owned();
            res = res.set_status(StatusCode::ImATeapot);
            res.set_body(format!("Teapot Status: {}", msg));
        }
        Ok(res)
    }));

    app.at("/").get(|_req: Request<_>| async move {
        let path = url::Url::parse("")?;
        Ok(format!("Path is {}", path))
    });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
