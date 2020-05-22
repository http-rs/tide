use crate::log;
use crate::{Body, Endpoint, Request, Response, Result, StatusCode};

use async_std::fs::File;
use async_std::io::BufReader;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'a + Send>>;
pub struct ServeDir {
    prefix: String,
    dir: PathBuf,
}

impl ServeDir {
    /// Create a new instance of `ServeDir`.
    pub(crate) fn new(prefix: String, dir: PathBuf) -> Self {
        Self { prefix, dir }
    }
}

impl<State> Endpoint<State> for ServeDir {
    fn call<'a>(&'a self, req: Request<State>) -> BoxFuture<'a, Result> {
        let path = req.uri().path();
        let path = path.trim_start_matches(&self.prefix);
        let path = path.trim_start_matches('/');
        let mut file_path = self.dir.clone();
        for p in Path::new(path) {
            if p == OsStr::new(".") {
                continue;
            } else if p == OsStr::new("..") {
                file_path.pop();
            } else {
                file_path.push(&p);
            }
        }

        log::info!("Requested file: {:?}", file_path);

        Box::pin(async move {
            if !file_path.starts_with(&self.dir) {
                log::warn!("Unauthorized attempt to read: {:?}", file_path);
                return Ok(Response::new(StatusCode::Forbidden));
            }

            let file = match File::open(&file_path).await {
                Err(error) => {
                    return if error.kind() == async_std::io::ErrorKind::NotFound {
                        log::warn!("File not found: {:?}", file_path);
                        Ok(Response::new(StatusCode::NotFound))
                    } else {
                        log::warn!("Could not open {:?}", file_path);
                        Ok(Response::new(StatusCode::InternalServerError))
                    }
                }
                Ok(file) => file,
            };

            let len = if let Ok(metadata) = file.metadata().await {
                metadata.len() as usize
            } else {
                log::warn!("Could not retrieve metadata");
                return Ok(Response::new(StatusCode::InternalServerError));
            };

            let body = Body::from_reader(BufReader::new(file), Some(len));
            // TODO: fix related bug where async-h1 crashes on large files
            let mut res = Response::new(StatusCode::Ok);
            res.set_body(body);

            if let Some(content_type) = mime_guess::from_path(&file_path).first() {
                res = res.set_mime(content_type);
            }

            Ok(res)
        })
    }
}
