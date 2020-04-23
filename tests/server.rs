mod test_utils;
use async_std::prelude::*;
use async_std::task;
use http_types::StatusCode;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[test]
fn hello_world() -> Result<(), http_types::Error> {
    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = tide::new();
            app.at("/").get(|mut req: tide::Request<()>| async move {
                assert_eq!(req.body_string().await.unwrap(), "nori".to_string());
                let res = tide::Response::new(StatusCode::Ok).body_string("says hello".to_string());
                Ok(res)
            });
            app.listen(&port).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::get(format!("http://{}", port))
                .body_string("nori".to_string())
                .recv_string()
                .await?;
            assert_eq!(string, "says hello".to_string());
            Ok(())
        });

        server.race(client).await
    })
}

#[test]
fn echo_server() -> Result<(), http_types::Error> {
    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = tide::new();
            app.at("/").get(|req| async move { Ok(req) });

            app.listen(&port).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let string = surf::get(format!("http://{}", port))
                .body_string("chashu".to_string())
                .recv_string()
                .await?;
            assert_eq!(string, "chashu".to_string());
            Ok(())
        });

        server.race(client).await
    })
}

#[test]
fn json() -> Result<(), http_types::Error> {
    #[derive(Deserialize, Serialize)]
    struct Counter {
        count: usize,
    }

    task::block_on(async {
        let port = test_utils::find_port().await;
        let server = task::spawn(async move {
            let mut app = tide::new();
            app.at("/").get(|mut req: tide::Request<()>| async move {
                let mut counter: Counter = req.body_json().await.unwrap();
                assert_eq!(counter.count, 0);
                counter.count = 1;
                let res = tide::Response::new(StatusCode::Ok).body_json(&counter)?;
                Ok(res)
            });
            app.listen(&port).await?;
            Result::<(), http_types::Error>::Ok(())
        });

        let client = task::spawn(async move {
            task::sleep(Duration::from_millis(100)).await;
            let counter: Counter = surf::get(format!("http://{}", &port))
                .body_json(&Counter { count: 0 })?
                .recv_json()
                .await?;
            assert_eq!(counter.count, 1);
            Ok(())
        });

        server.race(client).await
    })
}
