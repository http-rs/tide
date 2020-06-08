use tide::utils::After;
use tide::{Body, Request, Response, Result, StatusCode};

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();
    let mut app = tide::new();

    app.at("/")
        .middleware(After(|mut res: Response| async {
            if let Some(err) = res.downcast_error::<async_std::io::Error>() {
                let msg = err.to_string().to_owned();
                res.set_status(StatusCode::ImATeapot);
                res.set_body(format!("Teapot Status: {}", msg));
            }
            Ok(res)
        }))
        .get(|_req: Request<_>| async {
            let mut res = Response::new(StatusCode::Ok);
            res.set_body(Body::from_file("./does-not-exist").await?);
            Ok(res)
        });

    app.at("/uncaught").get(|_req: Request<_>| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(Body::from_file("./does-not-exist").await?);
        Ok(res)
    });

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
