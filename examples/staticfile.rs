#![feature(async_await)]

use futures_fs::FsPool;
use futures_util::compat::*;
use http::{header, response::Builder, Response as HTTPResponse, StatusCode};
use http_service::Body;
use tide::{App, Context, EndpointResult, Response};

use std::path::{Component, Path, PathBuf};
use std::{
    fs::{self, Metadata},
    io,
};

const DEFAULT_4XX_BODY: &[u8] = b"Oops! I can't find what you're looking for...";
const DEFAULT_5XX_BODY: &[u8] = b"I'm broken, apparently.";

type StaticFileResult<T> = Result<T, io::Error>;

/// Simple static file handler for Tide inspired from https://github.com/iron/staticfile.
#[derive(Clone)]
struct StaticFile {
    fs_pool: FsPool,
    root: PathBuf,
}

impl StaticFile {
    /// Percent-decode, normalize path components and return the final path joined with root.
    /// See https://github.com/iron/staticfile/blob/master/src/requested_path.rs
    fn normalize(path: &str) -> PathBuf {
        Path::new(path)
            .components()
            .fold(PathBuf::new(), |mut result, p| {
                match p {
                    Component::Normal(x) => {
                        if let Some(s) = x.to_str() {
                            if !s.is_empty() {
                                result.push({
                                    &*percent_encoding::percent_decode(s.as_bytes())
                                        .decode_utf8_lossy()
                                });
                            }
                        }
                    }
                    Component::ParentDir => {
                        result.pop();
                    }
                    _ => (),
                }

                result
            })
    }

    pub fn new(path: impl AsRef<Path>) -> Self {
        // normalize the root path to match all other normalized path elements
        let root = StaticFile::normalize(path.as_ref().to_str().unwrap());

        // TODO: Error handling: propagation, log or panic?
        if !root.exists() {
            eprintln!(
                "StaticFile path not found: {}",
                root.to_str().unwrap_or("Unknown")
            );
        }

        StaticFile {
            root,
            fs_pool: FsPool::default(),
        }
    }

    /// Guess mime type and finish the file stream response
    fn file_stream_res(
        &self,
        meta: Metadata,
        path: PathBuf,
        mut response: Builder,
    ) -> StaticFileResult<Response> {
        // TODO: More mime types support needed
        let mime = mime_guess::guess_mime_type(&path);
        let mime_str = mime.as_ref();
        let size = meta.len();

        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_str)
            .header(header::CONTENT_LENGTH, size);

        let stream = self.fs_pool.read(path.to_owned(), Default::default());

        Ok(response
            .body(Body::from_stream(stream.compat()))
            .expect("invalid request?"))
    }

    /// Serve directory as html
    /// TODO: Other content types like JSON?
    fn dir_res(
        &self,
        actual_path: &str,
        path: PathBuf,
        mut response: Builder,
    ) -> StaticFileResult<Response> {
        let mut files_html: Vec<String> = Vec::new();

        // create files html
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let name_str = &entry_path.file_name().unwrap().to_str().unwrap();
            let link = String::from(actual_path) + name_str;
            let metadata = fs::metadata(&entry_path)?;
            let icon_str = if entry_path.is_dir() {
                "&#x25A0;"
            } else {
                "&#x25EA;"
            };

            // file html string
            files_html.push(format!(
                r#"
                <tr>
                    <td><span class="icon">{}</span><a href="{}" title="{}">{}</a></td>
                    <td>{:.2} KB</td>
                </tr>
            "#,
                icon_str,
                link,
                link,
                name_str,
                metadata.len() as f64 / 1024_f64
            ));
        }

        // push a back-link if not inside of the top most directory
        if path != self.root {
            files_html.push(String::from(r#"<tr><td colspan="2"><span class="icon">&#x25AA;&#x25AA;</span><a href="../" title="back">Back</a></td></tr>"#));
        }

        // TODO: Group files_html by type?

        // TODO: should be probably a template or at least minified
        Ok(response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
            .body(format!(r#"
                <!DOCTYPE html>
                <html>
                    <head>
                        <meta charset="utf-8">
                        <title>{}</title>
                        <style>
                            html,body {{position:relative;width: 100%;height: 100%;background-color: #FEFEFC;}}
                            body {{color: #333;margin: 0;padding: 0;font-size: 14px;line-height: 18px;
                            word-wrap: break-word; font-family: -apple-system, BlinkMacSystemFont,
                            "Segoe UI", Roboto, Oxygen-Sans, Ubuntu, Cantarell, "Helvetica Neue", sans-serif;}}
                            * {{box-sizing: border-box;}}
                            .files {{border-collapse: collapse;table-layout: fixed;width: 100%;}}
                            .files a {{color: #176395; overflow: hidden; outline: none; box-shadow: none;
                            text-decoration: none; width: 100%; left; white-space: nowrap;text-overflow: ellipsis;}}
                            .files .icon {{color: #176395; font-size: 16px; text-align: center; margin-right: 15px;}}
                            .files a:visited {{color: #4793c5;}}
                            .files a:hover {{text-decoration: underline;}}
                            .files tbody tr{{background-color: white;}}
                            .files tbody tr:hover{{background-color: rgba(222,233,230,.2);}}
                            .files thead th{{color: #176395; background-color: rgba(0,0,0,.03);
                            border-left: 1px solid rgba(0,0,0,.05); border-bottom: 1px solid rgba(0,0,0,.05);}}
                            .files th,td {{ border-bottom: 1px solid rgba(0,0,0,.05); padding: 10px;
                            text-align: left; white-space: nowrap;text-overflow: ellipsis;}}
                            .files .size-col{{width: 250px;}}
                        </style>
                    </head>
                    <body>
                        <table class="files">
                            <thead>
                                <tr>
                                    <th>{}</th>
                                    <th class="size-col">Size</th>
                                </tr>
                            </thead>
                            <tbody>{}</tbody>
                        </table>
                    </body>
                </html>
            "#, actual_path, actual_path, files_html.join("")).into())
            .expect("failed to serve dir?"))
    }

    /// Finish 404-response
    fn not_found_res(mut response: Builder) -> StaticFileResult<Response> {
        Ok(response
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
            .body(DEFAULT_4XX_BODY.into())
            .expect("failed to build static response?"))
    }

    /// Prepend root path and normalize the path
    fn get_path(&self, path: &str) -> PathBuf {
        self.root.join(StaticFile::normalize(path))
    }

    /// Handles requested path based o its location and type
    fn handle_path(&self, actual_path: &str) -> StaticFileResult<Response> {
        let path = self.get_path(actual_path);
        let mut response = HTTPResponse::builder();

        // Prevent access outside the root path
        if path.starts_with(&self.root) {
            // if file exists
            if let Ok(meta) = fs::metadata(&path) {
                // if file is a directory
                if meta.is_dir() {
                    // if the path don't end with / redirect to path/
                    if !actual_path.ends_with('/') {
                        return Ok(response
                            .status(StatusCode::MOVED_PERMANENTLY)
                            .header(header::LOCATION, String::from(actual_path) + "/")
                            .body(Body::empty())
                            .expect("failed to build redirect response?"));
                    }

                    // serve index.html if it is inside requested directory
                    let index_path = Path::new(&path).join("index.html");
                    if let Ok(meta) = fs::metadata(&index_path) {
                        return self.file_stream_res(meta, index_path, response);
                    } else {
                        // serve the entire directory if there is no index.html
                        return self.dir_res(actual_path, path, response);
                    }
                }

                // stream a file that must exist and is not a directory
                return self.file_stream_res(meta, path, response);
            }
        }

        // nothing was found or the path was outside the root directory
        StaticFile::not_found_res(response)
    }
}

async fn serve_static_file(ctx: Context<StaticFile>) -> EndpointResult {
    let actual_path = ctx.uri().path();
    ctx.state().handle_path(actual_path).or_else(|_err| {
        Ok(http::Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
            .body(DEFAULT_5XX_BODY.into())
            .expect("failed to build static response?"))
    })
}

fn main() {
    let mut app = App::with_state(StaticFile::new("./examples/static"));
    app.at("").get(serve_static_file);
    app.at("/*").get(serve_static_file);
    app.run("127.0.0.1:8000").unwrap();
}
