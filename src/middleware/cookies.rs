use crate::{
    middleware::{Middleware, Next},
    response::CookieEvent,
    Request, Response,
};
use cookie::{Cookie, CookieJar, ParseError};
use futures::future::BoxFuture;

use http::HeaderMap;
use std::sync::{Arc, RwLock};

/// A middleware for making cookie data available in requests.
///
/// # Examples
///
/// ```rust
///
/// let mut app = tide::Server::new();
/// app.at("/get").get(|cx: tide::Request<()>| async move { cx.cookie("testCookie").unwrap().unwrap().value().to_string() });
/// app.at("/set").get(|_| async {
///     let mut res = tide::Response::new(200);
///     res.set_cookie(cookie::Cookie::new("testCookie", "NewCookieValue"));
///     res
/// });
///
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
    ) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let cookie_jar = if let Some(cookie_data) = ctx.local::<CookieData>() {
                cookie_data.content.clone()
            } else {
                // no cookie data in local context, so we need to create it
                let cookie_data = CookieData::from_headers(ctx.headers());
                let content = cookie_data.content.clone();
                ctx = ctx.set_local(cookie_data);
                content
            };

            let mut res = next.run(ctx).await;

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
                let set_cookie_header = http::header::SET_COOKIE.as_ref();
                let encoded_cookie = cookie.encoded().to_string();
                res = res.append_header(set_cookie_header, encoded_cookie);
            }
            res
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CookieData {
    pub(crate) content: Arc<RwLock<CookieJar>>,
}

impl CookieData {
    pub(crate) fn from_headers(headers: &HeaderMap) -> Self {
        let cookie_header = headers.get(http::header::COOKIE);
        let cookie_jar = cookie_header.and_then(|raw| {
            let mut jar = CookieJar::new();

            // as long as we have an ascii string this will start parsing the cookie
            if let Some(raw_str) = raw.to_str().ok() {
                raw_str
                    .split(';')
                    .try_for_each(|s| -> Result<_, ParseError> {
                        jar.add_original(Cookie::parse(s.trim().to_owned())?);
                        Ok(())
                    })
                    .ok()?;
            }

            Some(jar)
        });
        let content = Arc::new(RwLock::new(cookie_jar.unwrap_or_default()));

        CookieData { content }
    }
}
