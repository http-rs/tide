use crate::{error::Error, Request};
use http::StatusCode;
use serde::Deserialize;

/// An extension trait for `Request`, providing query string deserialization.
///
/// # Example
///
/// Turning the query parameters into a `HashMap`:
///
/// ```
/// # use std::collections::HashMap;
/// use tide::querystring::RequestExt;
///
/// let mut app = tide::Server::new();
/// app.at("/").get(|cx: tide::Request<()>| async move {
///     let map: HashMap<String, String> = cx.url_query().unwrap();
///     format!("{:?}", map)
/// });
/// ```
pub trait RequestExt<'de> {
    fn url_query<T: Deserialize<'de>>(&'de self) -> Result<T, Error>;
}

impl<'de, State> RequestExt<'de> for Request<State> {
    #[inline]
    fn url_query<T: Deserialize<'de>>(&'de self) -> Result<T, Error> {
        let query = self.uri().query();

        if query.is_none() {
            return Err(Error::from(StatusCode::BAD_REQUEST));
        }

        Ok(serde_qs::from_str(query.unwrap()).map_err(|_| Error::from(StatusCode::BAD_REQUEST))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use http_service::Body;
    use http_service_mock::make_server;
    use serde_derive::Deserialize;

    #[derive(Deserialize)]
    struct Params {
        msg: String,
    }

    async fn handler(cx: crate::Request<()>) -> Result<String, Error> {
        let p = cx.url_query::<Params>()?;
        Ok(p.msg)
    }

    fn app() -> crate::Server<()> {
        let mut app = crate::Server::new();
        app.at("/").get(handler);
        app
    }

    #[test]
    fn successfully_deserialize_query() {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get("/?msg=Hello")
            .body(Body::empty())
            .unwrap();
        let res = server.simulate(req).unwrap();
        assert_eq!(res.status(), 200);
        let body = block_on(res.into_body().into_vec()).unwrap();
        assert_eq!(&*body, &*b"Hello");
    }

    #[test]
    fn unsuccessfully_deserialize_query() {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get("/").body(Body::empty()).unwrap();
        let res = server.simulate(req).unwrap();
        assert_eq!(res.status(), 400);
    }

    #[test]
    fn malformatted_query() {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get("/?error=should_fail")
            .body(Body::empty())
            .unwrap();
        let res = server.simulate(req).unwrap();
        assert_eq!(res.status(), 400);
    }
}
