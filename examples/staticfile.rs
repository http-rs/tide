use bytes::Bytes;
use futures_fs::FsPool;
use futures_util::compat::*;
use http::{
    header::{self, HeaderMap},
    StatusCode,
};
use http_service::Body;
use tide::{App, Context, EndpointResult, Response};

use std::path::{Component, Path, PathBuf};
use std::{fs, io};

const DEFAULT_4XX_BODY: &[u8] = b"Oops! I can't find what you're looking for..." as &[_];
const DEFAULT_5XX_BODY: &[u8] = b"I'm broken, apparently." as &[_];

/// Simple static file handler for Tide inspired from https://github.com/iron/staticfile.
#[derive(Clone)]
struct StaticFile {
    fs_pool: FsPool,
    root: PathBuf,
}

impl StaticFile {
    /// Creates a new instance of this handler.
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = PathBuf::from(root.as_ref());
        if !root.exists() {
            // warn maybe?
        }

        StaticFile {
            root,
            fs_pool: FsPool::default(),
        }
    }

    fn stream_bytes(&self, actual_path: &str, headers: &HeaderMap) -> Result<Response, io::Error> {
        let path = &self.get_path(actual_path);
        let mut response = http::Response::builder();
        let meta = fs::metadata(path).ok();
        // Check if the path exists and handle if it's a directory containing `index.html`
        if meta.is_some() && meta.as_ref().map(|m| !m.is_file()).unwrap_or(false) {
            // Redirect if path is a dir and URL doesn't end with "/"
            if !actual_path.ends_with("/") {
                return Ok(response
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .header(header::LOCATION, String::from(actual_path) + "/")
                    .body(Body::empty())
                    .expect("failed to build redirect response?"));
            }

            let index = Path::new(actual_path).join("index.html");
            return self.stream_bytes(&*index.to_string_lossy(), headers);
        }

        // If the file doesn't exist, then bail out.
        let meta = match meta {
            Some(m) => m,
            None => {
                return Ok(response
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
                    .body(DEFAULT_4XX_BODY.into())
                    .expect("failed to build static response?"))
            }
        };

        let mime = mime_guess::guess_mime_type(path);
        let mime_str = mime.as_ref();
        let size = meta.len();

        // We're done with the checks. Stream file!
        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_str)
            .header(header::CONTENT_LENGTH, size);

        let stream = self.fs_pool.read(PathBuf::from(path), Default::default());
        Ok(response
            .body(Body::from_stream(stream.compat()))
            .expect("invalid request?"))
    }

    /// Percent-decode, normalize path components and return the final path joined with root.
    /// See https://github.com/iron/staticfile/blob/master/src/requested_path.rs
    fn get_path(&self, path: &str) -> PathBuf {
        let rel_path = Path::new(path)
            .components()
            .fold(PathBuf::new(), |mut result, p| {
                match p {
                    Component::Normal(x) => result.push({
                        let s = x.to_str().unwrap_or("");
                        &*percent_encoding::percent_decode(s.as_bytes()).decode_utf8_lossy()
                    }),
                    Component::ParentDir => {
                        result.pop();
                    }
                    _ => (), // ignore any other component
                }

                result
            });

        self.root.join(rel_path)
    }
}

async fn handle_path(ctx: Context<StaticFile>) -> EndpointResult {
    let path = ctx.uri().path();
    ctx.state()
        .stream_bytes(path, ctx.headers())
        .or_else(|_err| {
            Ok(http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
                .body(Bytes::from(DEFAULT_5XX_BODY).into())
                .expect("failed to build static response?"))
        })
}

fn main() {
    let mut app = App::with_state(StaticFile::new("./"));
    app.at("/*").get(handle_path);
    app.serve("127.0.0.1:8000").unwrap();
}
