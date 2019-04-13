use futures::future::FutureObj;
use std::sync::Arc;

use http::{
    header::{HeaderValue, IntoHeaderName},
    HeaderMap, HttpTryFrom,
};

use cookie::{Cookie, CookieJar, ParseError};
use crate::cookies::CookieData;
use std::borrow::Cow;

use crate::{
    middleware::{Middleware, Next},
    Context, Response,
};

#[derive(Clone, Default)]
pub struct CookiesMiddleware {
}

impl CookiesMiddleware {
    pub fn new() -> Self {
        Self{}
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for CookiesMiddleware {
    fn handle<'a>(&'a self, mut cx: Context<Data>, next: Next<'a, Data>) -> FutureObj<'a, Response> {
        box_async! {
            let cookie_data = cx
            .extensions_mut()
            .remove()
            .unwrap_or_else(|| CookieData::from_headers(cx.headers()));

            let cookie_jar = cookie_data.content.clone();

            let mut res = await!(next.run(cx));
            let mut headers = HeaderMap::new();
            for cookie in cookie_jar.delta() {
                let hv = HeaderValue::from_str(cookie.encoded().to_string().as_str());
                match hv {
                    Ok(val) => { headers.insert("Set-Cookie", val); },
                    Err(_) => {
                        //log error here
                    },
                };
            }
            res.headers_mut().extend(headers);
            res
        }
    }
}