use crate::data::CookieData;
use futures::future::BoxFuture;
use http::header::HeaderValue;

use tide_core::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// Middleware to work with cookies.
///
/// [`CookiesMiddleware`] along with [`ContextExt`] provide smooth
/// access to request cookies and setting/removing cookies from response. This leverages the
/// [cookie](https://crates.io/crates/cookie) crate.
/// This middleware parses cookies from request and caches them in the extension. Once the request
/// is processed by endpoints and other middlewares, all the added and removed cookies are set on
/// on the response. You will need to add this middle before any other middlewares that might need
/// to access Cookies.
///
/// [`CookiesMiddleware`]: crate::middleware::CookiesMiddleware
/// [`ContextExt`]: ../../tide/cookies/trait.ContextExt.html
#[derive(Clone, Default, Debug)]
pub struct CookiesMiddleware {}

impl CookiesMiddleware {
    /// CookieMiddleware constructor
    pub fn new() -> Self {
        Self {}
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for CookiesMiddleware {
    fn handle<'a>(
        &'a self,
        mut cx: Context<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let cookie_data = cx
                .extensions_mut()
                .remove()
                .unwrap_or_else(|| CookieData::from_headers(cx.headers()));

            let cookie_jar = cookie_data.content.clone();

            // The `let _ = ...` is a workaround for issue: https://github.com/rustasync/tide/issues/278
            // Solution is according to suggestion in https://github.com/rust-lang/rust/issues/61579#issuecomment-500436524
            let _ = cx.extensions_mut().insert(cookie_data);
            let mut res = next.run(cx).await;
            let headers = res.headers_mut();
            for cookie in cookie_jar.read().unwrap().delta() {
                let hv = HeaderValue::from_str(cookie.encoded().to_string().as_str());
                if let Ok(val) = hv {
                    headers.append(http::header::SET_COOKIE, val);
                } else {
                    // TODO It would be useful to log this error here.
                    return http::Response::builder()
                        .status(http::status::StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "text/plain; charset=utf-8")
                        .body(http_service::Body::empty())
                        .unwrap();
                }
            }
            res
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ContextExt;
    use cookie::Cookie;
    use futures::executor::block_on;
    use http_service::Body;
    use http_service_mock::make_server;
    use tide_core::Context;

    static COOKIE_NAME: &str = "testCookie";

    /// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
    async fn retrieve_cookie(mut cx: Context<()>) -> String {
        cx.get_cookie(COOKIE_NAME)
            .unwrap()
            .unwrap()
            .value()
            .to_string()
    }

    async fn set_cookie(mut cx: Context<()>) {
        cx.set_cookie(Cookie::new(COOKIE_NAME, "NewCookieValue"))
            .unwrap();
    }

    async fn remove_cookie(mut cx: Context<()>) {
        cx.remove_cookie(Cookie::named(COOKIE_NAME)).unwrap();
    }

    async fn set_multiple_cookie(mut cx: Context<()>) {
        cx.set_cookie(Cookie::new("C1", "V1")).unwrap();
        cx.set_cookie(Cookie::new("C2", "V2")).unwrap();
    }

    fn app() -> tide::App<()> {
        let mut app = tide::App::new();
        app.middleware(CookiesMiddleware::new());

        app.at("/get").get(retrieve_cookie);
        app.at("/set").get(set_cookie);
        app.at("/remove").get(remove_cookie);
        app.at("/multi").get(set_multiple_cookie);
        app
    }

    fn make_request(endpoint: &str) -> Response {
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
        let res = make_request("/get");
        assert_eq!(res.status(), 200);
        let body = block_on(res.into_body().into_vec()).unwrap();
        assert_eq!(&*body, &*b"RequestCookieValue");
    }

    #[test]
    fn successfully_set_cookie() {
        let res = make_request("/set");
        assert_eq!(res.status(), 204);
        let test_cookie_header = res.headers().get(http::header::SET_COOKIE).unwrap();
        assert_eq!(
            test_cookie_header.to_str().unwrap(),
            "testCookie=NewCookieValue"
        );
    }

    #[test]
    fn successfully_remove_cookie() {
        let res = make_request("/remove");
        assert_eq!(res.status(), 204);
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
        assert_eq!(res.status(), 204);
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
