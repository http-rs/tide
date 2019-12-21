use crate::{
    middleware::{Middleware, Next},
    Request, Response,
};
use cookie::{Cookie, CookieJar, ParseError};
use futures::future::BoxFuture;

use crate::Error;
use http::{status::StatusCode, HeaderMap};
use std::sync::{Arc, RwLock};

const MIDDLEWARE_MISSING_MSG: &str =
    "CookiesMiddleware must be used to populate request and response cookies";

/// A simple requests logger
///
/// # Examples
///
/// ```rust
///
/// let mut app = tide::Server::new();
/// app.middleware(tide::middleware::CookiesMiddleware::new());
/// ```
#[derive(Debug, Clone, Default)]
pub struct CookiesMiddleware;

impl CookiesMiddleware {
    pub fn new() -> Self {
        Self::default()
    }

    async fn handle_cookie<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> Response {
        let mut ctx = ctx;

        let cookie_jar = if let Some(cookie_data) = ctx.local::<CookieData>() {
            cookie_data.content.clone()
        } else {
            let cookie_data = CookieData::from_headers(ctx.headers());
            let cloned_content = cookie_data.content.clone(); // TODO: hmm does look ugly but needed because of moved cookie_data
            ctx = ctx.set_local(cookie_data);
            cloned_content
        };

        let mut res = next.run(ctx).await;

        for cookie in cookie_jar.read().unwrap().delta() {
            res = res.append_header(
                http::header::SET_COOKIE.as_ref(),
                cookie.encoded().to_string(),
            );
        }
        res
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for CookiesMiddleware {
    fn handle<'a>(&'a self, ctx: Request<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        Box::pin(async move { self.handle_cookie(ctx, next).await })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CookieData {
    pub(crate) content: Arc<RwLock<CookieJar>>,
}

impl CookieData {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        CookieData {
            content: Arc::new(RwLock::new(
                headers
                    .get(http::header::COOKIE)
                    .and_then(|raw| parse_from_header(raw.to_str().unwrap()).ok())
                    .unwrap_or_default(),
            )),
        }
    }
}

fn parse_from_header(s: &str) -> Result<CookieJar, ParseError> {
    let mut jar = CookieJar::new();

    s.split(';').try_for_each(|s| -> Result<_, ParseError> {
        jar.add_original(Cookie::parse(s.trim().to_owned())?);
        Ok(())
    })?;

    Ok(jar)
}

/// An extension to `Context` that provides cached access to cookies
pub trait RequestExt {
    /// returns a `Cookie` by name of the cookie
    fn get_cookie(&self, name: &str) -> Result<Option<Cookie<'static>>, Error>;

    /// Add cookie to the cookie jar
    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), Error>;

    /// Removes the cookie. This instructs the `CookiesMiddleware` to send a cookie with empty value
    /// in the response.
    fn remove_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), Error>;
}

impl<State> RequestExt for Request<State> {
    fn get_cookie(&self, name: &str) -> Result<Option<Cookie<'static>>, Error> {
        let cookie_data = self
            .local::<CookieData>()
            .ok_or_else(|| Error::from(StatusCode::INTERNAL_SERVER_ERROR))?;

        let locked_jar = cookie_data
            .content
            .read()
            .map_err(|e| Error::from(StatusCode::INTERNAL_SERVER_ERROR))?; // better mapping here
                                                                           // .map_err(|e| StringError(format!("Failed to get write lock: {}", e)))?;
        Ok(locked_jar.get(name).cloned())
    }

    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), Error> {
        let cookie_data = self
            .local::<CookieData>()
            .ok_or_else(|| Error::from(StatusCode::INTERNAL_SERVER_ERROR))?;

        let mut locked_jar = cookie_data
            .content
            .write()
            .map_err(|e| Error::from(StatusCode::INTERNAL_SERVER_ERROR))?; // better mapping here
        locked_jar.add(cookie);
        Ok(())
    }

    fn remove_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), Error> {
        let cookie_data = self
            .local::<CookieData>()
            .ok_or_else(|| Error::from(StatusCode::INTERNAL_SERVER_ERROR))?;

        let mut locked_jar = cookie_data
            .content
            .write()
            .map_err(|e| Error::from(StatusCode::INTERNAL_SERVER_ERROR))?; // better mapping here
        locked_jar.remove(cookie);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cookie::Cookie;
    use futures::executor::block_on;
    use futures::AsyncReadExt;
    use http_service::Body;
    use http_service_mock::make_server;

    static COOKIE_NAME: &str = "testCookie";

    /// Tide will use the the  `Cookies`'s `Extract` implementation to build this parameter.
    async fn retrieve_cookie(cx: Request<()>) -> String {
        cx.get_cookie(COOKIE_NAME)
            .unwrap()
            .unwrap()
            .value()
            .to_string()
    }

    async fn set_cookie(mut cx: Request<()>) -> Response {
        cx.set_cookie(Cookie::new(COOKIE_NAME, "NewCookieValue"))
            .unwrap();
        Response::new(200)
    }

    async fn remove_cookie(mut cx: Request<()>) -> Response {
        cx.remove_cookie(Cookie::named(COOKIE_NAME)).unwrap();
        Response::new(200)
    }

    async fn set_multiple_cookie(mut cx: Request<()>) -> Response {
        cx.set_cookie(Cookie::new("C1", "V1")).unwrap();
        cx.set_cookie(Cookie::new("C2", "V2")).unwrap();
        Response::new(200)
    }

    fn app() -> crate::Server<()> {
        let mut app = crate::new();
        app.middleware(CookiesMiddleware::new());

        app.at("/get").get(retrieve_cookie);
        app.at("/set").get(set_cookie);
        app.at("/remove").get(remove_cookie);
        app.at("/multi").get(set_multiple_cookie);
        app
    }

    fn make_request(endpoint: &str) -> http::response::Response<http_service::Body> {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get(endpoint)
            .header(http::header::COOKIE, "testCookie=RequestCookieValue")
            .body(Body::empty())
            .unwrap();
        server.simulate(req).unwrap()
    }

    #[test]
    fn successfully_retrieve_request_cookie() {
        let mut res = make_request("/get");
        assert_eq!(res.status(), 200);

        let body = block_on(async move {
            let mut buffer = Vec::new(); // init the buffer to read the data into
            res.body_mut().read_to_end(&mut buffer).await.unwrap();
            buffer
        });

        assert_eq!(&*body, &*b"RequestCookieValue");
    }

    #[test]
    fn successfully_set_cookie() {
        let res = make_request("/set");
        assert_eq!(res.status(), 200);
        let test_cookie_header = res.headers().get(http::header::SET_COOKIE).unwrap();
        assert_eq!(
            test_cookie_header.to_str().unwrap(),
            "testCookie=NewCookieValue"
        );
    }

    #[test]
    fn successfully_remove_cookie() {
        let res = make_request("/remove");
        assert_eq!(res.status(), 200);
        let test_cookie_header = res.headers().get(http::header::SET_COOKIE).unwrap();
        assert!(test_cookie_header
            .to_str()
            .unwrap()
            .starts_with("testCookie=;"));
        let cookie = Cookie::parse_encoded(test_cookie_header.to_str().unwrap()).unwrap();
        assert_eq!(cookie.name(), COOKIE_NAME);
        assert_eq!(cookie.value(), "");
        assert_eq!(cookie.http_only(), None);
        assert_eq!(cookie.max_age().unwrap().num_nanoseconds(), Some(0));
    }

    #[test]
    fn successfully_set_multiple_cookies() {
        let res = make_request("/multi");
        assert_eq!(res.status(), 200);
        let cookie_header = res.headers().get_all(http::header::SET_COOKIE);
        let mut iter = cookie_header.iter();

        let cookie1 = iter.next().unwrap();
        let cookie2 = iter.next().unwrap();

        //Headers can be out of order
        if cookie1.to_str().unwrap().starts_with("C1") {
            assert_eq!(cookie1, "C1=V1");
            assert_eq!(cookie2, "C2=V2");
        } else {
            assert_eq!(cookie2, "C1=V1");
            assert_eq!(cookie1, "C2=V2");
        }

        assert!(iter.next().is_none());
    }
}
