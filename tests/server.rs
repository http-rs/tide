mod test_utils;
use async_std::task;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tide::{Body, Request};

#[async_std::test]
async fn hello_world() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.at("/").get(move |mut req: Request<()>| async move {
            assert_eq!(req.body_string().await.unwrap(), "nori".to_string());
            assert!(req.local_addr().unwrap().contains(&port.to_string()));
            assert!(req.peer_addr().is_some());
            Ok("says hello")
        });
        app.listen(("localhost", port)).await
    });

    task::sleep(Duration::from_millis(100)).await;
    let string = surf::get(format!("http://localhost:{}", port))
        .body_string("nori".to_string())
        .recv_string()
        .await
        .unwrap();
    assert_eq!(string, "says hello");

    server.cancel().await;
    Ok(())
}

#[async_std::test]
async fn echo_server() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.at("/").get(|req| async move { Ok(req) });

        app.listen(("localhost", port)).await
    });

    task::sleep(Duration::from_millis(100)).await;
    let string = surf::get(format!("http://localhost:{}", port))
        .body_string("chashu".to_string())
        .recv_string()
        .await
        .unwrap();
    assert_eq!(string, "chashu".to_string());

    server.cancel().await;
    Ok(())
}

#[async_std::test]
async fn json() -> Result<(), http_types::Error> {
    #[derive(Deserialize, Serialize)]
    struct Counter {
        count: usize,
    }

    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.at("/").get(|mut req: Request<()>| async move {
            let mut counter: Counter = req.body_json().await.unwrap();
            assert_eq!(counter.count, 0);
            counter.count = 1;
            Ok(Body::from_json(&counter)?)
        });
        app.listen(("localhost", port)).await
    });

    task::sleep(Duration::from_millis(100)).await;
    let counter: Counter = surf::get(format!("http://localhost:{}", &port))
        .body_json(&Counter { count: 0 })?
        .recv_json()
        .await
        .unwrap();
    assert_eq!(counter.count, 1);

    server.cancel().await;
    Ok(())
}
