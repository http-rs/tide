use crate::response::CookieEvent;
use crate::utils::BoxFuture;
use crate::{Middleware, Next, Request};

use cookie::{Cookie, CookieJar};
use http_types::headers;

use std::sync::{Arc, RwLock};

/// A middleware for making cookie data available in requests.
///
/// # Examples
///
/// ```
/// # use tide::{Request, Response, StatusCode};
/// let mut app = tide::Server::new();
/// app.at("/get").get(|cx: Request<()>| async move { Ok(cx.cookie("testCookie").unwrap().value().to_string()) });
/// app.at("/set").get(|_| async {
///     let mut res = Response::new(StatusCode::Ok);
///     res.set_cookie(cookie::Cookie::new("testCookie", "NewCookieValue"));
///     Ok(res)
/// });
/// ```
#[derive(Debug, Clone, Default)]
pub(crate) struct CookiesMiddleware;

impl CookiesMiddleware {
    /// Creates a new CookiesMiddleware.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for CookiesMiddleware {
    fn handle<'a>(
        &'a self,
        mut ctx: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result> {
        Box::pin(async move {
            let cookie_jar = if let Some(cookie_data) = ctx.local::<CookieData>() {
                cookie_data.content.clone()
            } else {
                // no cookie data in local context, so we need to create it
                let cookie_data = CookieData::from_request(&ctx);
                let content = cookie_data.content.clone();
                ctx = ctx.set_local(cookie_data);
                content
            };

            let mut res = next.run(ctx).await?;

            // add modifications from response to original
            for cookie in res.cookie_events.drain(..) {
                match cookie {
                    CookieEvent::Added(cookie) => cookie_jar.write().unwrap().add(cookie.clone()),
                    CookieEvent::Removed(cookie) => {
                        cookie_jar.write().unwrap().remove(cookie.clone())
                    }
                }
            }

            // iterate over added and removed cookies
            for cookie in cookie_jar.read().unwrap().delta() {
                let encoded_cookie = cookie.encoded().to_string();
                res = res.append_header(headers::SET_COOKIE, encoded_cookie);
            }
            Ok(res)
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CookieData {
    pub(crate) content: Arc<RwLock<CookieJar>>,
}

impl CookieData {
    pub(crate) fn from_request<S>(req: &Request<S>) -> Self {
        let mut jar = CookieJar::new();

        if let Some(cookie_headers) = req.header(&headers::COOKIE) {
            for cookie_header in cookie_headers {
                // spec says there should be only one, so this is permissive
                for pair in cookie_header.as_str().split(";") {
                    if let Ok(cookie) = Cookie::parse_encoded(String::from(pair)) {
                        jar.add_original(cookie);
                    }
                }
            }
        }

        CookieData {
            content: Arc::new(RwLock::new(jar)),
        }
    }
}
