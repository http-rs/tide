use crate::cookies::CookieData;
use futures::future::BoxFuture;
use http::header::HeaderValue;

use crate::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// Middleware to work with cookies.
///
/// [`CookiesMiddleware`] along with [`CookiesExt`](crate::cookies::CookiesExt) provide smooth
/// access to request cookies and setting/removing cookies from response. This leverages the
/// [cookie](https://crates.io/crates/cookie) crate.
/// This middleware parses cookies from request and caches them in the extension. Once the request
/// is processed by endpoints and other middlewares, all the added and removed cookies are set on
/// on the response. You will need to add this middle before any other middlewares that might need
/// to access Cookies.
#[derive(Clone, Default, Debug)]
pub struct CookiesMiddleware {}

impl CookiesMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for CookiesMiddleware {
    fn handle<'a>(
        &'a self,
        mut cx: Context<Data>,
        next: Next<'a, Data>,
    ) -> BoxFuture<'a, Response> {
        box_async! {
            let cookie_data = cx
                .extensions_mut()
                .remove()
                .unwrap_or_else(|| CookieData::from_headers(cx.headers()));

            let cookie_jar = cookie_data.content.clone();

            cx.extensions_mut().insert(cookie_data);
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cookies::CookiesExt, Context};
    use cookie::Cookie;
    use futures::executor::block_on;
    use http_service::Body;
    use http_service_mock::make_server;

    static COOKIE_NAME: &str = "testCookie";

    /// Tide will use the the `Cookies`'s `Extract` implementation to build this parameter.
    #[allow(unused_mut)] // Workaround clippy bug
    async fn retrieve_cookie(mut cx: Context<()>) -> String {
        format!("{}", cx.get_cookie(COOKIE_NAME).unwrap().unwrap().value())
    }

    #[allow(unused_mut)] // Workaround clippy bug
    async fn set_cookie(mut cx: Context<()>) {
        cx.set_cookie(Cookie::new(COOKIE_NAME, "NewCookieValue"))
            .unwrap();
    }

    #[allow(unused_mut)] // Workaround clippy bug
    async fn remove_cookie(mut cx: Context<()>) {
        cx.remove_cookie(Cookie::named(COOKIE_NAME)).unwrap();
    }

    #[allow(unused_mut)] // Workaround clippy bug
    async fn set_multiple_cookie(mut cx: Context<()>) {
        cx.set_cookie(Cookie::new("C1", "V1")).unwrap();
        cx.set_cookie(Cookie::new("C2", "V2")).unwrap();
    }

    fn app() -> crate::App<()> {
        let mut app = crate::App::new();
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
        let res = server.simulate(req).unwrap();
        res
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
