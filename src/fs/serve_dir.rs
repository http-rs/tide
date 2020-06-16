use crate::log;
use crate::utils::BoxFuture;
use crate::{Body, Endpoint, Request, Response, Result, StatusCode};

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// An endpoint for serving static files from a base local file system
/// `dir`, mounted at a given route `prefix`.  As an example,
/// `ServeDir::new("/static", "./build/assets")` would resolve a
/// request to `/static/index.html` to
/// `(pwd)/build/assets/index.html`.
///
/// If that file did not exist, ServeDir would send a 404 Not found by default.
#[derive(Debug, Clone)]
pub struct ServeDir {
    prefix: String,
    dir: PathBuf,
}

impl ServeDir {
    /// Create a new instance of `ServeDir`.
    pub fn new(prefix: impl Into<String>, dir: impl Into<PathBuf>) -> Self {
        Self {
            prefix: prefix.into(),
            dir: dir.into(),
        }
    }

    /// Resolves a local filesystem path from a request's url path,
    /// taking the ServeDir's route `prefix` into consideration. If
    /// the path is not within the ServeDir's `dir`, this method
    /// returns None.
    pub fn resolve_path<State>(&self, request: &Request<State>) -> Option<PathBuf> {
        let path = request
            .url()
            .path()
            .trim_start_matches(&self.prefix)
            .trim_start_matches('/');

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

        if file_path.starts_with(&self.dir) {
            log::info!("Requested file: {:?}", file_path);
            Some(file_path)
        } else {
            log::warn!("Unauthorized attempt to read: {:?}", file_path);
            None
        }
    }
}

impl<State> Endpoint<State> for ServeDir {
    fn call<'a>(&'a self, req: Request<State>) -> BoxFuture<'a, Result> {
        let file_path = self.resolve_path(&req);
        Box::pin(async move {
            if let Some(file_path) = file_path {
                if let Ok(body) = Body::from_file(&file_path).await {
                    Ok(body.into())
                } else {
                    Ok(Response::new(StatusCode::NotFound))
                }
            } else {
                Ok(Response::new(StatusCode::Forbidden))
            }
        })
    }
}
