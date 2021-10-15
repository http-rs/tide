use crate::response::CookieEvent;
use crate::{Middleware, Next, Request, State};
use async_trait::async_trait;

use crate::http::cookies::{Cookie, CookieJar, Delta};
use crate::http::headers;

use std::sync::{Arc, RwLock};

/// A middleware for making cookie data available in requests.
///
/// # Examples
///
/// ```
/// # use tide::{Request, Response, StatusCode};
/// # use tide::http::cookies::Cookie;
/// let mut app = tide::Server::new();
/// app.at("/get").get(|req: Request, _| async move {
///     Ok(req.cookie("testCookie").unwrap().value().to_string())
/// });
/// app.at("/set").get(|_, _| async {
///     let mut res = Response::new(StatusCode::Ok);
///     res.insert_cookie(Cookie::new("testCookie", "NewCookieValue"));
///     Ok(res)
/// });
/// ```
#[derive(Debug, Clone, Default)]
pub(crate) struct CookiesMiddleware;

impl CookiesMiddleware {
    /// Creates a new `CookiesMiddleware`.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl<ServerState> Middleware<ServerState> for CookiesMiddleware
where
    ServerState: Clone + Send + Sync + 'static,
{
    async fn handle(
        &self,
        mut req: Request,
        state: State<ServerState>,
        next: Next<'_, ServerState>,
    ) -> crate::Result {
        let cookie_jar = if let Some(cookie_data) = req.ext::<CookieData>() {
            cookie_data.content.clone()
        } else {
            let cookie_data = CookieData::from_request(&req, &state);
            // no cookie data in ext context, so we try to create it
            let content = cookie_data.content.clone();
            req.set_ext(cookie_data);
            content
        };

        let mut res = next.run(req, state).await;

        // Don't do anything if there are no cookies.
        if res.cookie_events.is_empty() {
            return Ok(res);
        }

        let jar = &mut *cookie_jar.write().unwrap();

        // add modifications from response to original
        for cookie in res.cookie_events.drain(..) {
            match cookie {
                CookieEvent::Added(cookie) => jar.add(cookie.clone()),
                CookieEvent::Removed(cookie) => jar.remove(cookie.clone()),
            }
        }

        // iterate over added and removed cookies
        for cookie in jar.delta() {
            let encoded_cookie = cookie.encoded().to_string();
            res.append_header(headers::SET_COOKIE, encoded_cookie);
        }
        Ok(res)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct CookieData {
    pub(crate) content: Arc<RwLock<LazyJar>>,
}

#[derive(Debug, Default, Clone)]
/// Wrapper around `CookieJar`, that initializes only when actually used.
pub(crate) struct LazyJar(Option<CookieJar>);

impl LazyJar {
    fn add(&mut self, cookie: Cookie<'static>) {
        self.get_jar().add(cookie)
    }

    fn remove(&mut self, cookie: Cookie<'static>) {
        self.get_jar().remove(cookie)
    }

    fn delta(&mut self) -> Delta<'_> {
        self.get_jar().delta()
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        if let Some(jar) = &self.0 {
            return jar.get(name);
        }
        None
    }

    fn get_jar(&mut self) -> &mut CookieJar {
        if self.0.is_none() {
            self.0 = Some(CookieJar::new());
        }

        self.0.as_mut().unwrap()
    }
}

impl CookieData {
    pub(crate) fn from_request<S>(req: &Request, _state: &S) -> Self {
        let jar = if let Some(cookie_headers) = req.header(&headers::COOKIE) {
            let mut jar = CookieJar::new();
            for cookie_header in cookie_headers {
                // spec says there should be only one, so this is permissive
                for pair in cookie_header.as_str().split(';') {
                    if let Ok(cookie) = Cookie::parse_encoded(String::from(pair)) {
                        jar.add_original(cookie);
                    }
                }
            }

            LazyJar(Some(jar))
        } else {
            LazyJar::default()
        };

        CookieData {
            content: Arc::new(RwLock::new(jar)),
        }
    }
}
