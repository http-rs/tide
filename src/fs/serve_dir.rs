use crate::log;
use crate::{Body, Endpoint, Request, Response, Result, StatusCode};

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

impl<State> Endpoint<State> for ServeDir
where
    State: Send + Sync + 'static,
{
    fn call<'a>(&'a self, req: Request<State>) -> BoxFuture<'a, Result> {
        let path = req.url().path();
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
            let body = Body::from_file(&file_path).await?;
            let mut res = Response::new(StatusCode::Ok);
            res.set_body(body);
            Ok(res)
        })
    }
}
