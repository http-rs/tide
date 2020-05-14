//! Tide app test.
//!
//! # Examples
//!
//! ```no_run
//! use tide::{Request, Response, Server};
//!
//! #[async_std::main]
//! async fn main() -> std::io::Result<()> {
//!     get_app().listen("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//!
//! fn get_app() -> Server<()> {
//!     let mut app = tide::new();
//!     app.at("/hello").get(hello);
//!
//!     app
//! }
//!
//! async fn hello(_: Request<()>) -> tide::Result<Response> {
//!     Ok("Hello, world!".into())
//! }
//!
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use tide::server_test::ServerTest as _;
//!
//!     #[async_std::test]
//!     async fn test_hello() -> tide::Result<()> {
//!         let mut res = get_app().test_get("/hello").await?;
//!         assert_eq!(res.take_body().into_string().await?, "Hello, world!");
//!         Ok(())
//!     }
//! }
//! ```

use http_service::HttpService;
use std::convert::TryInto;

use crate::http;
use crate::utils::BoxFuture;
use crate::Server;

pub trait ServerTest {
    fn test_with_http_request<'a>(
        &'a self,
        http_req: http_service::Request,
    ) -> BoxFuture<'a, crate::Result<crate::Response>>;

    fn test_get<'a>(&'a self, path: &'a str) -> BoxFuture<'a, crate::Result<crate::Response>> {
        self.test_get_with_headers(path, Vec::<(&str, &str)>::new())
    }

    fn test_get_with_headers<'a>(
        &'a self,
        path: &'a str,
        headers: Vec<(
            impl TryInto<http::headers::HeaderName> + Send + Sync + 'a,
            impl http::headers::ToHeaderValues + Send + Sync + 'a,
        )>,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        match build_http_request(http::Method::Get, path, Option::<&str>::None, Some(headers)) {
            Ok(req) => self.test_with_http_request(req),
            Err(e) => Box::pin(async { Err(e) }),
        }
    }

    fn test_head<'a>(&'a self, path: &'a str) -> BoxFuture<'a, crate::Result<crate::Response>> {
        self.test_head_with_headers(path, Vec::<(&str, &str)>::new())
    }

    fn test_head_with_headers<'a>(
        &'a self,
        path: &'a str,
        headers: Vec<(
            impl TryInto<http::headers::HeaderName> + Send + Sync + 'a,
            impl http::headers::ToHeaderValues + Send + Sync + 'a,
        )>,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        match build_http_request(http::Method::Get, path, Option::<&str>::None, Some(headers)) {
            Ok(req) => self.test_with_http_request(req),
            Err(e) => Box::pin(async { Err(e) }),
        }
    }

    fn test_put<'a>(
        &'a self,
        path: &'a str,
        body: impl Into<http::Body> + Send + Sync + 'a,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        self.test_put_with_headers(path, body, Vec::<(&str, &str)>::new())
    }

    fn test_put_with_headers<'a>(
        &'a self,
        path: &'a str,
        body: impl Into<http::Body> + Send + Sync + 'a,
        headers: Vec<(
            impl TryInto<http::headers::HeaderName> + Send + Sync + 'a,
            impl http::headers::ToHeaderValues + Send + Sync + 'a,
        )>,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        match build_http_request(http::Method::Put, path, Some(body), Some(headers)) {
            Ok(req) => self.test_with_http_request(req),
            Err(e) => Box::pin(async { Err(e) }),
        }
    }

    fn test_post<'a>(
        &'a self,
        path: &'a str,
        body: impl Into<http::Body> + Send + Sync + 'a,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        self.test_post_with_headers(path, body, Vec::<(&str, &str)>::new())
    }

    fn test_post_with_headers<'a>(
        &'a self,
        path: &'a str,
        body: impl Into<http::Body> + Send + Sync + 'a,
        headers: Vec<(
            impl TryInto<http::headers::HeaderName> + Send + Sync + 'a,
            impl http::headers::ToHeaderValues + Send + Sync + 'a,
        )>,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        match build_http_request(http::Method::Post, path, Some(body), Some(headers)) {
            Ok(req) => self.test_with_http_request(req),
            Err(e) => Box::pin(async { Err(e) }),
        }
    }

    fn test_delete<'a>(&'a self, path: &'a str) -> BoxFuture<'a, crate::Result<crate::Response>> {
        self.test_delete_with_headers(path, Vec::<(&str, &str)>::new())
    }

    fn test_delete_with_headers<'a>(
        &'a self,
        path: &'a str,
        headers: Vec<(
            impl TryInto<http::headers::HeaderName> + Send + Sync + 'a,
            impl http::headers::ToHeaderValues + Send + Sync + 'a,
        )>,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        match build_http_request(
            http::Method::Delete,
            path,
            Option::<&str>::None,
            Some(headers),
        ) {
            Ok(req) => self.test_with_http_request(req),
            Err(e) => Box::pin(async { Err(e) }),
        }
    }

    fn test_options<'a>(&'a self, path: &'a str) -> BoxFuture<'a, crate::Result<crate::Response>> {
        self.test_options_with_headers(path, Vec::<(&str, &str)>::new())
    }

    fn test_options_with_headers<'a>(
        &'a self,
        path: &'a str,
        headers: Vec<(
            impl TryInto<http::headers::HeaderName> + Send + Sync + 'a,
            impl http::headers::ToHeaderValues + Send + Sync + 'a,
        )>,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        match build_http_request(
            http::Method::Options,
            path,
            Option::<&str>::None,
            Some(headers),
        ) {
            Ok(req) => self.test_with_http_request(req),
            Err(e) => Box::pin(async { Err(e) }),
        }
    }

    fn test_patch<'a>(
        &'a self,
        path: &'a str,
        body: impl Into<http::Body> + Send + Sync + 'a,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        self.test_patch_with_headers(path, body, Vec::<(&str, &str)>::new())
    }

    fn test_patch_with_headers<'a>(
        &'a self,
        path: &'a str,
        body: impl Into<http::Body> + Send + Sync + 'a,
        headers: Vec<(
            impl TryInto<http::headers::HeaderName> + Send + Sync + 'a,
            impl http::headers::ToHeaderValues + Send + Sync + 'a,
        )>,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        match build_http_request(http::Method::Patch, path, Some(body), Some(headers)) {
            Ok(req) => self.test_with_http_request(req),
            Err(e) => Box::pin(async { Err(e) }),
        }
    }
}

impl<State: Send + Sync + 'static> ServerTest for Server<State> {
    fn test_with_http_request<'a>(
        &'a self,
        http_req: http_service::Request,
    ) -> BoxFuture<'a, crate::Result<crate::Response>> {
        Box::pin(async move {
            self.respond((), http_req)
                .await
                .map(|res| crate::Response::from(res))
        })
    }
}

fn build_http_request<'a>(
    http_method: http::Method,
    path: &'a str,
    body: Option<impl Into<http::Body> + Send + Sync + 'a>,
    headers: Option<
        Vec<(
            impl TryInto<http::headers::HeaderName> + Send + Sync + 'a,
            impl http::headers::ToHeaderValues + Send + Sync + 'a,
        )>,
    >,
) -> crate::Result<http_service::Request> {
    let url = if path.starts_with("http://") || path.starts_with("https://") {
        path.to_owned()
    } else {
        format!("http://localhost/{}", path.trim_start_matches("/"))
    };
    let url = http::Url::parse(url.as_str())?;
    let mut http_req = http::Request::new(http_method, url);

    if let Some(body) = body {
        http_req.set_body(body);
    }

    if let Some(headers) = headers {
        for (name, values) in headers {
            http_req.insert_header(name, values)?;
        }
    }

    Ok(http_req)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::http;
    use crate::{self as tide};

    fn get_app() -> tide::Server<()> {
        let mut app = tide::new();
        app.at("/anything").get(anything).post(anything);

        app
    }

    async fn anything(mut req: tide::Request<()>) -> tide::Result<tide::Response> {
        let body_string = req.body_string().await?;
        let header_foo = req
            .header(&"foo".parse().unwrap())
            .map(|v| v.iter().map(|v| v.as_str()).collect())
            .unwrap_or(vec![])
            .join(",");
        Ok(format!(
            "method:{},url:{},body_string:{},header_foo:{}",
            req.method(),
            req.uri(),
            body_string,
            header_foo,
        )
        .into())
    }

    #[async_std::test]
    async fn test_with_http_request() -> tide::Result<()> {
        let mut res = get_app()
            .test_with_http_request(http::Request::new(
                http::Method::Get,
                http::Url::parse("http://localhost/anything?__a=1")?,
            ))
            .await?;
        assert_eq!(res.status(), http::StatusCode::Ok);
        assert_eq!(
            res.take_body().into_string().await?,
            "method:GET,url:http://localhost/anything?__a=1,body_string:,header_foo:"
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_get() -> tide::Result<()> {
        let mut res = get_app().test_get("/anything").await?;
        assert_eq!(res.status(), http::StatusCode::Ok);
        assert_eq!(
            res.take_body().into_string().await?,
            "method:GET,url:http://localhost/anything,body_string:,header_foo:"
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_get_with_headers() -> tide::Result<()> {
        let mut res = get_app()
            .test_get_with_headers("/anything", vec![("foo", "bar")])
            .await?;
        assert_eq!(res.status(), http::StatusCode::Ok);
        assert_eq!(
            res.take_body().into_string().await?,
            "method:GET,url:http://localhost/anything,body_string:,header_foo:bar"
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_post() -> tide::Result<()> {
        let mut res = get_app().test_post("/anything", "body").await?;
        assert_eq!(res.status(), http::StatusCode::Ok);
        assert_eq!(
            res.take_body().into_string().await?,
            "method:POST,url:http://localhost/anything,body_string:body,header_foo:"
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_post_with_headers() -> tide::Result<()> {
        let mut res = get_app()
            .test_post_with_headers("/anything", "body", vec![("foo", "bar")])
            .await?;
        assert_eq!(res.status(), http::StatusCode::Ok);
        assert_eq!(
            res.take_body().into_string().await?,
            "method:POST,url:http://localhost/anything,body_string:body,header_foo:bar"
        );

        Ok(())
    }
}
