use crate::{
    middleware::{Middleware, Next},
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
            let content = cookie_data.content.clone();
            ctx = ctx.set_local(cookie_data);
            content
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
    pub(crate) fn from_headers(headers: &HeaderMap) -> Self {
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
