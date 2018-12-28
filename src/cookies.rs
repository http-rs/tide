use cookie::Cookie;
use cookie::CookieJar;
use cookie::ParseError;
use futures::future;

use crate::response::IntoResponse;
use crate::{Extract, Request, Response, RouteMatch};

#[derive(Clone, Debug)]
pub struct Cookies {
    content: CookieJar,
}

impl Cookies {
    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        self.content.get(name)
    }
}

impl<S: 'static> Extract<S> for Cookies {
    // Note: cannot use `existential type` here due to ICE
    type Fut = future::Ready<Result<Self, Response>>;

    fn extract(data: &mut S, req: &mut Request, params: &Option<RouteMatch<'_>>) -> Self::Fut {
        match req.headers().get("Cookie") {
            Some(raw_cookies) => parse_from_header(raw_cookies.to_str().unwrap())
                .map(|t| future::ok(Cookies { content: t }))
                .unwrap_or_else(|err| {
                    future::err(http::status::StatusCode::BAD_REQUEST.into_response())
                }),
            _ => future::ok(Cookies {
                content: CookieJar::new(),
            }),
        }
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
