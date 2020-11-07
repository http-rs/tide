mod test_utils;
use async_std::task;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tide::{Body, Request};

#[test]
fn hello_world() -> tide::Result<()> {
    task::block_on(async {

        let cancelation_token = tide::CancelationToken::new();

        let port = test_utils::find_port().await;

        let mut app = tide::new();
        app.at("/").get(move |mut req: Request<()>| async move {
            assert_eq!(req.body_string().await.unwrap(), "nori".to_string());
            assert!(req.local_addr().unwrap().contains(&port.to_string()));
            assert!(req.peer_addr().is_some());
            Ok("says hello")
        });

        let server = app.listen_with_cancelation_token(("localhost", port), cancelation_token.clone());
        let server = task::spawn(server);

        task::sleep(Duration::from_millis(100)).await;
        let string = surf::get(format!("http://localhost:{}", port))
            .body(Body::from_string("nori".to_string()))
            .recv_string()
            .await
            .unwrap();
        assert_eq!(string, "says hello");

        cancelation_token.complete();

        server.await.expect("Server did not complete gracefully");
        Result::<(), http_types::Error>::Ok(())
    })
}

#[test]
fn echo_server() -> tide::Result<()> {
    task::block_on(async {

        let cancelation_token = tide::CancelationToken::new();

        let port = test_utils::find_port().await;
        let mut app = tide::new();
        app.at("/").get(|req| async move { Ok(req) });

        let server = app.listen_with_cancelation_token(("localhost", port), cancelation_token.clone());
        let server = task::spawn(server);

        task::sleep(Duration::from_millis(100)).await;
        let string = surf::get(format!("http://localhost:{}", port))
            .body(Body::from_string("chashu".to_string()))
            .recv_string()
            .await
            .unwrap();
        assert_eq!(string, "chashu".to_string());

        cancelation_token.complete();

        server.await.expect("Server did not complete gracefully");
        Result::<(), http_types::Error>::Ok(())
    })
}

#[test]
fn json() -> tide::Result<()> {
    #[derive(Deserialize, Serialize)]
    struct Counter {
        count: usize,
    }

    task::block_on(async {

        let cancelation_token = tide::CancelationToken::new();

        let port = test_utils::find_port().await;

        let mut app = tide::new();
        app.at("/").get(|mut req: Request<()>| async move {
            let mut counter: Counter = req.body_json().await.unwrap();
            assert_eq!(counter.count, 0);
            counter.count = 1;
            Ok(Body::from_json(&counter)?)
        });

        let server = app.listen_with_cancelation_token(("localhost", port), cancelation_token.clone());
        let server = task::spawn(server);

        task::sleep(Duration::from_millis(100)).await;
        let counter: Counter = surf::get(format!("http://localhost:{}", &port))
            .body(Body::from_json(&Counter { count: 0 })?)
            .recv_json()
            .await
            .unwrap();
        assert_eq!(counter.count, 1);

        server.await.expect("Server did not complete gracefully");
        Result::<(), http_types::Error>::Ok(())
    })
}
