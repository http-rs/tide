use async_std::task;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
struct Cat {
    name: String
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    task::block_on(async {
        let app = tide::new();

        app.at("/submit").post(|req: tide::Request<()>| async {
            let cat: Cat = req.body_json().await.unwrap();
            println!("cat name: {}", cat.name);

            let cat = Cat { name: "chashu".into() };
            let res = tide::Response::new(200).body_json(&cat).unwrap();
            res
        });

        app.listen("127.0.0.1:8080").await?;
        Ok(())
    })
}
