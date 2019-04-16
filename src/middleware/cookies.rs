use crate::cookies::CookieData;
use futures::future::FutureObj;
use http::header::HeaderValue;

use crate::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// `CookiesMiddleware middleware is required for `CookiesExt` implementation on `Context` to work.
/// This middleware parses cookies from request and caches them in the extension. Once the request
/// is processed by endpoints and other middlewares, all the added and removed cookies are set on
/// on the respone. You will need to add this middle before any other middlewares that might need to
/// access Cookies.
#[derive(Clone, Default)]
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
    ) -> FutureObj<'a, Response> {
        box_async! {
            let cookie_data = cx
                .extensions_mut()
                .remove()
                .unwrap_or_else(|| CookieData::from_headers(cx.headers()));

            let cookie_jar = cookie_data.content.clone();

            cx.extensions_mut().insert(cookie_data);
            let mut res = await!(next.run(cx));
            let headers = res.headers_mut();
            for cookie in cookie_jar.read().unwrap().delta() {
                let hv = HeaderValue::from_str(cookie.encoded().to_string().as_str());
                if let Ok(val) = hv {
                    headers.append(http::header::SET_COOKIE, val);
                } else {
                    // It would be useful to log this error here.
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
