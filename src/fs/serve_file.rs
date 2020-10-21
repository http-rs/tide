use crate::log;
use crate::{Body, Endpoint, Request, Response, Result, StatusCode};
use std::io;
use std::path::Path;

use async_std::path::PathBuf as AsyncPathBuf;
use async_trait::async_trait;

pub struct ServeFile {
    file: AsyncPathBuf,
}

impl ServeFile {
    /// Create a new instance of `ServeFile`.
    pub(crate) fn init(file: impl AsRef<Path>) -> io::Result<Self> {
        let file = file.as_ref().to_owned().canonicalize()?;
        Ok(Self {
            file: AsyncPathBuf::from(file),
        })
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Endpoint<State> for ServeFile {
    async fn call(&self, _: Request<State>) -> Result {
        if !self.file.exists().await {
            log::warn!("File not found: {:?}", &self.file);
            println!("File not found: {:?}", &self.file);
            Ok(Response::new(StatusCode::NotFound))
        } else {
            Ok(Response::builder(StatusCode::Ok)
                .body(Body::from_file(&self.file).await?)
                .build())
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
            file: AsyncPathBuf::from("gone/file"),
        };

        let res: Response = serve_file.call(request("static/foo")).await.unwrap().into();

        assert_eq!(res.status(), 404);
    }
}
