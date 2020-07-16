use std::sync::Arc;

use async_std::{fs::OpenOptions, io};
use tempfile::TempDir;
use tide::prelude::*;
use tide::{Body, Request, Response, StatusCode};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::with_state(Arc::new(tempfile::tempdir()?));

    // To test this example:
    // $ cargo run --example upload
    // $ curl -T ./README.md locahost:8080 # this writes the file to a temp directory
    // $ curl localhost:8080/README.md # this reads the file from the same temp directory

    app.at(":file")
        .put(|req: Request<Arc<TempDir>>| async move {
            let path: String = req.param("file")?;
            let fs_path = req.state().path().join(path);

            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&fs_path)
                .await?;

            let bytes_written = io::copy(req, file).await?;

            tide::log::info!("file written", {
                bytes: bytes_written,
                path: fs_path.canonicalize()?.to_str()
            });

            Ok(json!({ "bytes": bytes_written }))
        })
        .get(|req: Request<Arc<TempDir>>| async move {
            let path: String = req.param("file")?;
            let fs_path = req.state().path().join(path);

            if let Ok(body) = Body::from_file(fs_path).await {
                Ok(body.into())
            } else {
                Ok(Response::new(StatusCode::NotFound))
            }
        });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
