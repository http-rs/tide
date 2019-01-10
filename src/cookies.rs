use cookie::{Cookie, CookieJar, ParseError};
use futures::future;

use crate::response::IntoResponse;
use crate::{Extract, Request, Response, RouteMatch};

#[derive(Clone, Debug)]
pub struct Cookies {
    content: CookieJar,
}

impl Cookies {
    #[inline]
    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.content.get(name)
    }
}

impl<S: 'static> Extract<S> for Cookies {
    type Fut = future::Ready<Result<Self, Response>>;

    fn extract(data: &mut S, req: &mut Request, params: &Option<RouteMatch<'_>>) -> Self::Fut {
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
