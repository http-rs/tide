#[cfg(unix)]
mod unix_tests {
    use async_std::os::unix::net::UnixStream;
    use async_std::prelude::*;
    use async_std::task;
    use http_types::{url::Url, Method, Request, Response, StatusCode};
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn hello_unix_world() -> Result<(), http_types::Error> {
        task::block_on(async {
            let tmp_dir = tempdir()?;
            let sock_path = tmp_dir.path().join("sock");
            let sock_path_for_client = sock_path.clone();

            let server = task::spawn(async move {
                let mut app = tide::new();
                app.at("/").get(|req: tide::Request<()>| async move {
                    let mut res = Response::new(StatusCode::Ok);
                    res.set_body(serde_json::json!({
                        "peer_addr": req.peer_addr().unwrap(),
                        "local_addr": req.local_addr().unwrap()
                    }));
                    Ok(res)
                });
                app.listen_unix(sock_path).await?;
                http_types::Result::Ok(())
            });

            let client = task::spawn(async move {
                task::sleep(Duration::from_millis(100)).await;
                let listener = UnixStream::connect(&sock_path_for_client).await?;
                let req = Request::new(Method::Get, Url::parse("unix://local.socket/").unwrap());
                let mut res = async_h1::connect(listener, req).await?;
                let body: serde_json::Value = res.body_json().await.unwrap();
                assert!(body.get("peer_addr").unwrap().is_string());
                assert!(body
                    .get("local_addr")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .contains(sock_path_for_client.to_str().unwrap()));
                Ok(())
            });

            server.race(client).await
        })
    }
}
