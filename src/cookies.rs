use cookie::{Cookie, CookieJar, ParseError};
use futures::future;

use crate::{configuration::Store, response::IntoResponse, Extract, Request, Response, RouteMatch};

/// A representation of cookies which wraps `CookieJar` from `cookie` crate
///
/// Currently this only exposes getting cookie by name but future enhancements might allow more
/// operation. `Cookies` implements`Extract` so that handler methods can have a `Cookies` parameter.
///
#[derive(Clone, Debug)]
pub struct Cookies {
    content: CookieJar,
}

impl Cookies {
    /// returns a `Cookie` by name of the cookie
    #[inline]
    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.content.get(name)
    }
}

impl<S: 'static> Extract<S> for Cookies {
    type Fut = future::Ready<Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let cookie_jar = match req.headers().get("Cookie") {
            Some(raw_cookies) => parse_from_header(raw_cookies.to_str().unwrap()),
            _ => Ok(CookieJar::new()),
        };
        let resp = cookie_jar
            .map(|c| Cookies { content: c })
            .map_err(|_e| http::status::StatusCode::BAD_REQUEST.into_response());

        future::ready(resp)
    }
}

fn parse_from_header(s: &str) -> Result<CookieJar, ParseError> {
    let mut jar = CookieJar::new();

    s.split(';').try_for_each(|s| -> Result<_, ParseError> {
        jar.add(Cookie::parse(s.trim().to_owned())?);

        Ok(())
    })?;

    Ok(jar)
}
