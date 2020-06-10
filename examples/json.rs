use async_std::task;
use serde::{Deserialize, Serialize};
use tide::prelude::*;
use tide::{Body, Request};

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

            Ok(Body::from_json(&cat)?)
        });

        app.at("/animals").get(|_| async {
            Ok(json!({
                "meta": { "count": 2 },
                "animals": [
                    { "type": "cat", "name": "chashu" },
                    { "type": "cat", "name": "nori" }
                ]
            }))
        });

        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
