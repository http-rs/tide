use crate::cookies::CookieData;
use futures::future::FutureObj;
use http::header::HeaderValue;

use crate::{
    middleware::{Middleware, Next},
    Context, Response,
};

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
                 };
                 // TODO: raise error in case of Error?
            }
            res
        }
    }
}
