use async_std::task;
use serde::{Deserialize, Serialize};
use tide::prelude::*;
use tide::{Body, Request, Response, Result};

#[derive(Deserialize, Serialize)]
struct Cat {
    name: String,
}

fn main() -> tide::Result<()> {
    task::block_on(async {
        let mut app = tide::new();

        app.at("/submit").post(|mut req: Request<()>| async move {
            let cat: Cat = req.body_json().await?;
            println!("cat name: {}", cat.name);

            let cat = Cat {
                name: "chashu".into(),
            };

            let mut res = Response::new(200);
            res.set_body(Body::from_json(&cat)?);

            Result::Ok(res)
        });

        app.at("/animals").get(|_| async {
            json!({
                "meta": { "count": 2 },
                "animals": [
                    { "type": "cat", "name": "chashu" },
                    { "type": "cat", "name": "nori" }
                ]
            })
        });

        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
