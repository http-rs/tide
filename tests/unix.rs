#[cfg(unix)]
mod unix_tests {
    use async_std::os::unix::net::UnixStream;
    use async_std::prelude::*;
    use async_std::task;
    use http_types::{url::Url, Method, Request};
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
                    Ok(req.local_addr().unwrap().to_string())
                });
                app.listen(sock_path).await?;
                http_types::Result::Ok(())
            });

            let client = task::spawn(async move {
                task::sleep(Duration::from_millis(100)).await;
                let listener = UnixStream::connect(&sock_path_for_client).await?;
                let req = Request::new(Method::Get, Url::parse("http://local.socket/").unwrap());
                let mut res = async_h1::connect(listener, req).await?;
                let local_addr = res.body_string().await?;
                assert_eq!(
                    local_addr,
                    format!(
                        "http+unix://{}",
                        sock_path_for_client.canonicalize()?.to_str().unwrap()
                    )
                );
                Ok(())
            });

            server.race(client).await
        })
    }
}
