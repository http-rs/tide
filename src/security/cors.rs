use http_types::headers::{HeaderValue, HeaderValues};
use http_types::{headers, Method, StatusCode};
use regex::Regex;
use std::hash::Hash;

use crate::middleware::{Middleware, Next};
use crate::{Request, Result};

/// Middleware for CORS
///
/// # Example
///
/// ```no_run
/// use http_types::headers::HeaderValue;
/// use tide::security::{CorsMiddleware, Origin};
///
/// let cors = CorsMiddleware::new()
///     .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
///     .allow_origin(Origin::from("*"))
///     .allow_credentials(false);
/// ```
#[derive(Clone, Debug, Hash)]
pub struct CorsMiddleware {
    allow_credentials: Option<HeaderValue>,
    allow_headers: HeaderValue,
    allow_methods: HeaderValue,
    allow_origin: Origin,
    expose_headers: Option<HeaderValue>,
    max_age: HeaderValue,
}

pub(crate) const DEFAULT_MAX_AGE: &str = "86400";
pub(crate) const DEFAULT_METHODS: &str = "GET, POST, OPTIONS";
pub(crate) const WILDCARD: &str = "*";

impl CorsMiddleware {
    /// Creates a new Cors middleware.
    #[must_use]
    pub fn new() -> Self {
        Self {
            allow_credentials: None,
            allow_headers: WILDCARD.parse().unwrap(),
            allow_methods: DEFAULT_METHODS.parse().unwrap(),
            allow_origin: Origin::Any,
            expose_headers: None,
            max_age: DEFAULT_MAX_AGE.parse().unwrap(),
        }
    }

    /// Set `allow_credentials` and return new Cors
    #[must_use]
    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.allow_credentials = match allow_credentials.to_string().parse() {
            Ok(header) => Some(header),
            Err(_) => None,
        };
        self
    }

    /// Set `allow_headers` and return new Cors
    pub fn allow_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.allow_headers = headers.into();
        self
    }

    /// Set `max_age` and return new Cors
    pub fn max_age<T: Into<HeaderValue>>(mut self, max_age: T) -> Self {
        self.max_age = max_age.into();
        self
    }

    /// Set `allow_methods` and return new Cors
    pub fn allow_methods<T: Into<HeaderValue>>(mut self, methods: T) -> Self {
        self.allow_methods = methods.into();
        self
    }

    /// Set `allow_origin` and return new Cors
    pub fn allow_origin<T: Into<Origin>>(mut self, origin: T) -> Self {
        self.allow_origin = origin.into();
        self
    }

    /// Set `expose_headers` and return new Cors
    pub fn expose_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.expose_headers = Some(headers.into());
        self
    }

    fn build_preflight_response(&self, origins: &HeaderValues) -> http_types::Response {
        let mut response = http_types::Response::new(StatusCode::Ok);
        response.insert_header(headers::ACCESS_CONTROL_ALLOW_ORIGIN, origins);

        response.insert_header(
            headers::ACCESS_CONTROL_ALLOW_METHODS,
            self.allow_methods.clone(),
        );

        response.insert_header(
            headers::ACCESS_CONTROL_ALLOW_HEADERS,
            self.allow_headers.clone(),
        );

        response.insert_header(headers::ACCESS_CONTROL_MAX_AGE, self.max_age.clone());

        if let Some(allow_credentials) = self.allow_credentials.clone() {
            response.insert_header(headers::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials);
        }

        if let Some(expose_headers) = self.expose_headers.clone() {
            response.insert_header(headers::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers);
        }

        response
    }

    /// Look at origin of request and determine `allow_origin`
    fn response_origin(&self, origin: &HeaderValue) -> HeaderValue {
        match self.allow_origin {
            Origin::Any => WILDCARD.parse().unwrap(),
            _ => origin.clone(),
        }
    }

    /// Determine if origin is appropriate
    fn is_valid_origin(&self, origin: &HeaderValue) -> bool {
        let origin = origin.as_str().to_string();

        match &self.allow_origin {
            Origin::Any => true,
            Origin::Exact(s) => s == &origin,
            Origin::List(list) => list.contains(&origin),
            Origin::Match(regex) => regex.is_match(&origin),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for CorsMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Result {
        // TODO: how should multiple origin values be handled?
        let origins = req.header(&headers::ORIGIN).cloned();

        if origins.is_none() {
            // This is not a CORS request if there is no Origin header
            return Ok(next.run(req).await);
        }

        let origins = origins.unwrap();
        let origin = origins.last();

        if !self.is_valid_origin(origin) {
            return Ok(http_types::Response::new(StatusCode::Unauthorized).into());
        }

        // Return results immediately upon preflight request
        if req.method() == Method::Options {
            return Ok(self.build_preflight_response(&origins).into());
        }

        let mut response = next.run(req).await;

        response.insert_header(
            headers::ACCESS_CONTROL_ALLOW_ORIGIN,
            self.response_origin(origin),
        );

        if let Some(allow_credentials) = &self.allow_credentials {
            response.insert_header(
                headers::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                allow_credentials.clone(),
            );
        }

        if let Some(expose_headers) = &self.expose_headers {
            response.insert_header(
                headers::ACCESS_CONTROL_EXPOSE_HEADERS,
                expose_headers.clone(),
            );
        }

        Ok(response)
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// `allow_origin` enum
#[derive(Clone, Debug)]
pub enum Origin {
    /// Wildcard. Accept all origin requests
    Any,
    /// Set a single allow_origin target
    Exact(String),
    /// Set multiple allow_origin targets
    List(Vec<String>),
    /// Set a regex allow_origin targets
    Match(Regex),
}

impl From<String> for Origin {
    fn from(s: String) -> Self {
        if s == "*" {
            return Self::Any;
        }
        Self::Exact(s)
    }
}

impl From<&str> for Origin {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<Vec<String>> for Origin {
    fn from(list: Vec<String>) -> Self {
        if list.len() == 1 {
            return Self::from(list[0].clone());
        }

        Self::List(list)
    }
}

impl From<Regex> for Origin {
    fn from(regex: Regex) -> Self {
        Self::Match(regex)
    }
}

impl From<Vec<&str>> for Origin {
    fn from(list: Vec<&str>) -> Self {
        Self::from(
            list.iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<String>>(),
        )
    }
}

impl PartialEq for Origin {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Exact(this), Self::Exact(other)) => this == other,
            (Self::List(this), Self::List(other)) => this == other,
            (Self::Match(this), Self::Match(other)) => this.to_string() == other.to_string(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Hash for Origin {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Any => core::mem::discriminant(self).hash(state),
            Self::Exact(s) => s.hash(state),
            Self::List(list) => list.hash(state),
            Self::Match(regex) => regex.to_string().hash(state),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use http_types::headers::{self, HeaderValue};

    const ALLOW_ORIGIN: &str = "example.com";
    const ALLOW_METHODS: &str = "GET, POST, OPTIONS, DELETE";
    const EXPOSE_HEADER: &str = "X-My-Custom-Header";

    const ENDPOINT: &str = "/cors";

    fn endpoint_url() -> http_types::Url {
        format!("http://{}{}", ALLOW_ORIGIN, ENDPOINT)
            .parse()
            .unwrap()
    }

    fn app() -> crate::Server<()> {
        let mut app = crate::Server::new();
        app.at(ENDPOINT).get(|_| async { Ok("Hello World") });

        app
    }

    fn request() -> http_types::Request {
        let mut req = http_types::Request::new(http_types::Method::Get, endpoint_url());
        req.insert_header(http_types::headers::ORIGIN, ALLOW_ORIGIN);
        req
    }

    #[async_std::test]
    async fn preflight_request() {
        let mut app = app();
        app.with(
            CorsMiddleware::new()
                .allow_origin(Origin::from(ALLOW_ORIGIN))
                .allow_methods(ALLOW_METHODS.parse::<HeaderValue>().unwrap())
                .expose_headers(EXPOSE_HEADER.parse::<HeaderValue>().unwrap())
                .allow_credentials(true),
        );

        let mut req = http_types::Request::new(http_types::Method::Options, endpoint_url());
        req.insert_header(http_types::headers::ORIGIN, ALLOW_ORIGIN);

        let res: crate::http::Response = app.respond(req).await.unwrap();

        assert_eq!(res.status(), 200);

        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_ORIGIN], ALLOW_ORIGIN);
        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_METHODS], ALLOW_METHODS);
        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_HEADERS], WILDCARD);
        assert_eq!(res[headers::ACCESS_CONTROL_MAX_AGE], DEFAULT_MAX_AGE);

        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_CREDENTIALS], "true");
    }
    #[async_std::test]
    async fn default_cors_middleware() {
        let mut app = app();
        app.with(CorsMiddleware::new());
        let res: crate::http::Response = app.respond(request()).await.unwrap();

        assert_eq!(res.status(), 200);
        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
    }

    #[async_std::test]
    async fn custom_cors_middleware() {
        let mut app = app();
        app.with(
            CorsMiddleware::new()
                .allow_origin(Origin::from(ALLOW_ORIGIN))
                .allow_credentials(false)
                .allow_methods(ALLOW_METHODS.parse::<HeaderValue>().unwrap())
                .expose_headers(EXPOSE_HEADER.parse::<HeaderValue>().unwrap()),
        );
        let res: crate::http::Response = app.respond(request()).await.unwrap();

        assert_eq!(res.status(), 200);
        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_ORIGIN], ALLOW_ORIGIN);
    }

    #[async_std::test]
    async fn regex_cors_middleware() {
        let regex = Regex::new(r"e[xzs]a.*le.com*").unwrap();
        let mut app = app();
        app.with(
            CorsMiddleware::new()
                .allow_origin(Origin::from(regex))
                .allow_credentials(false)
                .allow_methods(ALLOW_METHODS.parse::<HeaderValue>().unwrap())
                .expose_headers(EXPOSE_HEADER.parse::<HeaderValue>().unwrap()),
        );
        let res: crate::http::Response = app.respond(request()).await.unwrap();

        assert_eq!(res.status(), 200);
        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_ORIGIN], ALLOW_ORIGIN);
    }

    #[async_std::test]
    async fn credentials_true() {
        let mut app = app();
        app.with(CorsMiddleware::new().allow_credentials(true));
        let res: crate::http::Response = app.respond(request()).await.unwrap();

        assert_eq!(res.status(), 200);
        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_CREDENTIALS], "true");
    }

    #[async_std::test]
    async fn set_allow_origin_list() {
        let mut app = app();
        let origins = vec![ALLOW_ORIGIN, "foo.com", "bar.com"];
        app.with(CorsMiddleware::new().allow_origin(origins.clone()));

        for origin in origins {
            let mut req = http_types::Request::new(http_types::Method::Get, endpoint_url());
            req.insert_header(http_types::headers::ORIGIN, origin);

            let res: crate::http::Response = app.respond(req).await.unwrap();

            assert_eq!(res.status(), 200);
            assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_ORIGIN][0], origin);
        }
    }

    #[async_std::test]
    async fn not_set_origin_header() {
        let mut app = app();
        app.with(CorsMiddleware::new().allow_origin(ALLOW_ORIGIN));

        let req = crate::http::Request::new(http_types::Method::Get, endpoint_url());
        let res: crate::http::Response = app.respond(req).await.unwrap();

        assert_eq!(res.status(), 200);
    }

    #[async_std::test]
    async fn unauthorized_origin() {
        let mut app = app();
        app.with(CorsMiddleware::new().allow_origin(ALLOW_ORIGIN));

        let mut req = http_types::Request::new(http_types::Method::Get, endpoint_url());
        req.insert_header(http_types::headers::ORIGIN, "unauthorize-origin.net");
        let res: crate::http::Response = app.respond(req).await.unwrap();

        assert_eq!(res.status(), 401);
    }

    #[async_std::test]
    #[cfg(feature = "cookies")]
    async fn retain_cookies() {
        let mut app = crate::Server::new();
        app.with(CorsMiddleware::new().allow_origin(ALLOW_ORIGIN));
        app.at(ENDPOINT).get(|_| async {
            let mut res = crate::Response::new(http_types::StatusCode::Ok);
            res.insert_cookie(http_types::Cookie::new("foo", "bar"));
            Ok(res)
        });

        let mut req = http_types::Request::new(http_types::Method::Get, endpoint_url());
        req.insert_header(http_types::headers::ORIGIN, ALLOW_ORIGIN);
        let res: crate::http::Response = app.respond(req).await.unwrap();

        assert_eq!(res[http_types::headers::SET_COOKIE][0], "foo=bar");
    }

    #[async_std::test]
    async fn set_cors_headers_to_error_responses() {
        let mut app = crate::Server::new();
        app.at(ENDPOINT).get(|_| async {
            Err::<&str, _>(crate::Error::from_str(
                StatusCode::BadRequest,
                "bad request",
            ))
        });
        app.with(CorsMiddleware::new().allow_origin(Origin::from(ALLOW_ORIGIN)));

        let res: crate::http::Response = app.respond(request()).await.unwrap();
        assert_eq!(res.status(), 400);
        assert_eq!(res[headers::ACCESS_CONTROL_ALLOW_ORIGIN], ALLOW_ORIGIN);
    }

    #[cfg(test)]
    mod origin {
        use super::super::Origin;
        use regex::Regex;

        #[test]
        fn transitive() {
            let regex = Regex::new(r"e[xzs]a.*le.com*").unwrap();
            let x = Origin::from(regex.clone());
            let y = Origin::from(regex.clone());
            let z = Origin::from(regex);
            assert!(x == y && y == z && x == z);
        }

        #[test]
        #[allow(clippy::nonminimal_bool)]
        fn symetrical() {
            let regex = Regex::new(r"e[xzs]a.*le.com*").unwrap();
            let x = Origin::from(regex.clone());
            let y = Origin::from(regex);
            assert!(x == y && y == x);
        }

        #[test]
        fn reflexive() {
            let regex = Regex::new(r"e[xzs]a.*le.com*").unwrap();
            let x = Origin::from(regex);
            assert!(x == x);
        }
    }
}
