use cookie::{Cookie, CookieJar, ParseError};

use crate::Context;
use http::HeaderMap;
use std::sync::{Arc, RwLock};

/// A representation of cookies which wraps `CookieJar` from `cookie` crate
///
#[derive(Debug)]
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

/// An extension to `Context` that provides cached access to cookies
pub trait CookiesExt {
    /// returns a `Cookie` by name of the cookie
    fn get_cookie(&mut self, name: &str) -> Option<Cookie<'static>>;
    fn set_cookie(&mut self, cookie: Cookie<'static>);
    fn remove_cookie(&mut self, cookie: Cookie<'static>);
}

impl<AppData> CookiesExt for Context<AppData> {
    fn get_cookie(&mut self, name: &str) -> Option<Cookie<'static>> {
        let cookie_data = self.extensions().get::<CookieData>().unwrap();
        let arc_jar = cookie_data.content.clone();
        let jar = arc_jar.read().unwrap();
        jar.get(name).cloned()
    }

    fn set_cookie(&mut self, cookie: Cookie<'static>) {
        let cookie_data = self.extensions().get::<CookieData>().unwrap();
        let jar = cookie_data.content.clone();
        jar.write().unwrap().add(cookie);
    }

    fn remove_cookie(&mut self, cookie: Cookie<'static>) {
        let cookie_data = self.extensions().get::<CookieData>().unwrap();

        let jar = cookie_data.content.clone();
        jar.write().unwrap().remove(cookie);
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
