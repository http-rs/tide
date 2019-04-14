use cookie::{Cookie, CookieJar, ParseError};

use crate::error::StringError;
use crate::Context;
use http::HeaderMap;
use std::sync::{Arc, RwLock};

const MIDDLEWARE_MISSING_MSG: &str =
    "CookiesMiddleware must be used to populate request and response cookies";

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
    fn get_cookie(&mut self, name: &str) -> Result<Option<Cookie<'static>>, StringError>;
    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), StringError>;
    fn remove_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), StringError>;
}

impl<AppData> CookiesExt for Context<AppData> {
    fn get_cookie(&mut self, name: &str) -> Result<Option<Cookie<'static>>, StringError> {
        let cookie_data = self
            .extensions()
            .get::<CookieData>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let arc_jar = cookie_data.content.clone();
        let locked_jar = arc_jar
            .read()
            .map_err(|e| StringError(format!("Failed to get write lock: {}", e)))?;
        Ok(locked_jar.get(name).cloned())
    }

    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), StringError> {
        let cookie_data = self
            .extensions()
            .get::<CookieData>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;
        let jar = cookie_data.content.clone();
        let mut locked_jar = jar
            .write()
            .map_err(|e| StringError(format!("Failed to get write lock: {}", e)))?;
        locked_jar.add(cookie);
        Ok(())
    }

    fn remove_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), StringError> {
        let cookie_data = self
            .extensions()
            .get::<CookieData>()
            .ok_or_else(|| StringError(MIDDLEWARE_MISSING_MSG.to_owned()))?;

        let jar = cookie_data.content.clone();
        let mut locked_jar = jar
            .write()
            .map_err(|e| StringError(format!("Failed to get write lock: {}", e)))?;
        locked_jar.remove(cookie);
        Ok(())
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
