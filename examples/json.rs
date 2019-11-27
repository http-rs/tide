use async_std::io;
use async_std::task;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Cat {
    name: String,
}

fn main() -> io::Result<()> {
    task::block_on(async {
        let mut app = tide::new();

        app.at("/submit").post(|mut req: tide::Request<()>| {
            async move {
                let cat: Cat = req.body_json().await.unwrap();
                println!("cat name: {}", cat.name);

                let cat = Cat {
                    name: "chashu".into(),
                };
                tide::Response::new(200).body_json(&cat).unwrap()
            }
        });

        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
