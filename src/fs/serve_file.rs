use crate::{Body, Endpoint, Request, Response, Result, StatusCode};
use std::io;
use std::path::Path;

use async_std::path::PathBuf as AsyncPathBuf;
use async_trait::async_trait;
use kv_log_macro::warn;

pub(crate) struct ServeFile {
    path: AsyncPathBuf,
}

impl ServeFile {
    /// Create a new instance of `ServeFile`.
    pub(crate) fn init(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = path.as_ref().to_owned().canonicalize()?;
        Ok(Self {
            path: AsyncPathBuf::from(file),
        })
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Endpoint<State> for ServeFile {
    async fn call(&self, _: Request<State>) -> Result {
        match Body::from_file(&self.path).await {
            Ok(body) => Ok(Response::builder(StatusCode::Ok).body(body).build()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                warn!("File not found: {:?}", &self.path);
                Ok(Response::new(StatusCode::NotFound))
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::http::{Response, Url};
    use std::fs::{self, File};
    use std::io::Write;

    fn serve_file(tempdir: &tempfile::TempDir) -> crate::Result<ServeFile> {
        let static_dir = tempdir.path().join("static");
        fs::create_dir(&static_dir)?;

        let file_path = static_dir.join("foo");
        let mut file = File::create(&file_path)?;
        write!(file, "Foobar")?;

        Ok(ServeFile::init(file_path)?)
    }

    fn request(path: &str) -> crate::Request<()> {
        let request =
            crate::http::Request::get(Url::parse(&format!("http://localhost/{}", path)).unwrap());
        crate::Request::new((), request, vec![])
    }

    #[async_std::test]
    async fn should_serve_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let serve_file = serve_file(&tempdir).unwrap();

        let mut res: Response = serve_file.call(request("static/foo")).await.unwrap().into();

        assert_eq!(res.status(), 200);
        assert_eq!(res.body_string().await.unwrap(), "Foobar");
    }

    #[async_std::test]
    async fn should_serve_404_when_file_missing() {
        let serve_file = ServeFile {
            path: AsyncPathBuf::from("gone/file"),
        };

        let res: Response = serve_file.call(request("static/foo")).await.unwrap().into();

        assert_eq!(res.status(), 404);
    }
}
