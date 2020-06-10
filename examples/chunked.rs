use async_std::task;
use tide::{Body, Response, StatusCode};

fn main() -> Result<(), std::io::Error> {
    task::block_on(async {
        let mut app = tide::new();
        app.at("/").get(|_| async move {
            let mut res = Response::new(StatusCode::Ok);
            res.set_body(Body::from_file(file!()).await.unwrap());
            Ok(res)
        });
        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
