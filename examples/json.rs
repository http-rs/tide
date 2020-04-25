use async_std::io;
use async_std::task;
use serde::{Deserialize, Serialize};
use tide::{Request, Response, StatusCode};

#[derive(Deserialize, Serialize)]
struct Cat {
    name: String,
}

fn main() -> io::Result<()> {
    task::block_on(async {
        let mut app = tide::new();

        app.at("/submit").post(|mut req: Request<()>| async move {
            let cat: Cat = req.body_json().await?;
            println!("cat name: {}", cat.name);

            let cat = Cat {
                name: "chashu".into(),
            };

            Ok(Response::new(StatusCode::Ok).body_json(&cat)?)
        });

        app.listen("127.0.0.1:8080").await
    })
}
