use crate::utils::BoxFuture;
use http_types::headers::HeaderValue;
use http_types::{headers, Method, StatusCode};

use crate::middleware::{Middleware, Next};
use crate::{Request, Response, Result};

/// Middleware for CORS
///
/// # Example
///
/// ```no_run
/// use http_types::headers::HeaderValue;
/// use tide::security::{CorsMiddleware, Origin};
///
/// CorsMiddleware::new()
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

pub const DEFAULT_MAX_AGE: &str = "86400";
pub const DEFAULT_METHODS: &str = "GET, POST, OPTIONS";
pub const WILDCARD: &str = "*";

impl CorsMiddleware {
    /// Creates a new Cors middleware.
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

    /// Set allow_credentials and return new Cors
    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.allow_credentials = match allow_credentials.to_string().parse() {
            Ok(header) => Some(header),
            Err(_) => None,
        };
        self
    }

    /// Set allow_headers and return new Cors
    pub fn allow_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.allow_headers = headers.into();
        self
    }

    /// Set max_age and return new Cors
    pub fn max_age<T: Into<HeaderValue>>(mut self, max_age: T) -> Self {
        self.max_age = max_age.into();
        self
    }

    /// Set allow_methods and return new Cors
    pub fn allow_methods<T: Into<HeaderValue>>(mut self, methods: T) -> Self {
        self.allow_methods = methods.into();
        self
    }

    /// Set allow_origin and return new Cors
    pub fn allow_origin<T: Into<Origin>>(mut self, origin: T) -> Self {
        self.allow_origin = origin.into();
        self
    }

    /// Set expose_headers and return new Cors
    pub fn expose_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.expose_headers = Some(headers.into());
        self
    }

    fn build_preflight_response(&self, origin: &[HeaderValue]) -> http_types::Response {
        let mut response = http_types::Response::new(StatusCode::Ok);
        response
            .insert_header(headers::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone())
            .unwrap();
        response
            .insert_header(
                headers::ACCESS_CONTROL_ALLOW_METHODS,
                self.allow_methods.clone(),
            )
            .unwrap();
        response
            .insert_header(
                headers::ACCESS_CONTROL_ALLOW_HEADERS,
                self.allow_headers.clone(),
            )
            .unwrap();
        response
            .insert_header(headers::ACCESS_CONTROL_MAX_AGE, self.max_age.clone())
            .unwrap();

        if let Some(allow_credentials) = self.allow_credentials.clone() {
            response
                .insert_header(headers::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials)
                .unwrap();
        }

        if let Some(expose_headers) = self.expose_headers.clone() {
            response
                .insert_header(headers::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers)
                .unwrap();
        }

        response
    }

    /// Look at origin of request and determine allow_origin
    fn response_origin(&self, origin: &HeaderValue) -> Option<HeaderValue> {
        if !self.is_valid_origin(origin) {
            return None;
        }

        match self.allow_origin {
            Origin::Any => Some(WILDCARD.parse().unwrap()),
            _ => Some(origin.clone()),
        }
    }

    /// Determine if origin is appropriate
    fn is_valid_origin(&self, origin: &HeaderValue) -> bool {
        let origin = origin.as_str().to_string();

        match &self.allow_origin {
            Origin::Any => true,
            Origin::Exact(s) => s == &origin,
            Origin::List(list) => list.contains(&origin),
        }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for CorsMiddleware {
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Result<Response>> {
        Box::pin(async move {
            let origins = req.header(&headers::ORIGIN).cloned().unwrap_or_default();

            // TODO: how should multiple origin values be handled?
            let origin = match origins.first() {
                Some(origin) => origin,
                None => {
                    // This is not a CORS request if there is no Origin header
                    return next.run(req).await;
                }
            };

            if !self.is_valid_origin(origin) {
                return Ok(http_types::Response::new(StatusCode::Unauthorized).into());
            }

            // Return results immediately upon preflight request
            if req.method() == Method::Options {
                return Ok(self.build_preflight_response(&origins).into());
            }

            let mut response: http_service::Response = next.run(req).await?.into();
            response
                .insert_header(
                    headers::ACCESS_CONTROL_ALLOW_ORIGIN,
                    self.response_origin(&origin).unwrap(),
                )
                .unwrap();

            if let Some(allow_credentials) = self.allow_credentials.clone() {
                response
                    .insert_header(headers::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials)
                    .unwrap();
            }

            if let Some(expose_headers) = self.expose_headers.clone() {
                response
                    .insert_header(headers::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers)
                    .unwrap();
            }
            Ok(response.into())
        })
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// allow_origin enum
#[derive(Clone, Debug, Hash, PartialEq)]
pub enum Origin {
    /// Wildcard. Accept all origin requests
    Any,
    /// Set a single allow_origin target
    Exact(String),
    /// Set multiple allow_origin targets
    List(Vec<String>),
}

impl From<String> for Origin {
    fn from(s: String) -> Self {
        if s == "*" {
            return Origin::Any;
        }
        Origin::Exact(s)
    }
}

impl From<&str> for Origin {
    fn from(s: &str) -> Self {
        Origin::from(s.to_string())
    }
}

impl From<Vec<String>> for Origin {
    fn from(list: Vec<String>) -> Self {
        if list.len() == 1 {
            return Self::from(list[0].clone());
        }

        Origin::List(list)
    }
}

impl From<Vec<&str>> for Origin {
    fn from(list: Vec<&str>) -> Self {
        Origin::from(list.iter().map(|s| s.to_string()).collect::<Vec<String>>())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use http_service_mock::make_server;
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
        app.at(ENDPOINT).get(|_| async move { Ok("Hello World") });

        app
    }

    fn request() -> http_types::Request {
        let mut req = http_types::Request::new(http_types::Method::Get, endpoint_url());
        req.insert_header(http_types::headers::ORIGIN, ALLOW_ORIGIN)
            .unwrap();
        req
    }

    #[test]
    fn preflight_request() {
        let mut app = app();
        app.middleware(
            CorsMiddleware::new()
                .allow_origin(Origin::from(ALLOW_ORIGIN))
                .allow_methods(ALLOW_METHODS.parse::<HeaderValue>().unwrap())
                .expose_headers(EXPOSE_HEADER.parse::<HeaderValue>().unwrap())
                .allow_credentials(true),
        );

        let mut server = make_server(app).unwrap();

        let mut req = http_types::Request::new(http_types::Method::Options, endpoint_url());
        req.insert_header(http_types::headers::ORIGIN, ALLOW_ORIGIN)
            .unwrap();

        let res = server.simulate(req).unwrap();

        assert_eq!(res.status(), 200);

        assert_eq!(
            res.header(&headers::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap()[0].as_str(),
            ALLOW_ORIGIN
        );
        assert_eq!(
            res.header(&headers::ACCESS_CONTROL_ALLOW_METHODS).unwrap()[0].as_str(),
            ALLOW_METHODS
        );
        assert_eq!(
            res.header(&headers::ACCESS_CONTROL_ALLOW_HEADERS).unwrap()[0].as_str(),
            WILDCARD
        );
        assert_eq!(
            res.header(&headers::ACCESS_CONTROL_MAX_AGE).unwrap()[0].as_str(),
            DEFAULT_MAX_AGE
        );

        assert_eq!(
            res.header(&headers::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap()[0]
                .as_str(),
            "true"
        );
    }
    #[test]
    fn default_cors_middleware() {
        let mut app = app();
        app.middleware(CorsMiddleware::new());

        let mut server = make_server(app).unwrap();
        let res = server.simulate(request()).unwrap();

        assert_eq!(res.status(), 200);

        assert_eq!(
            res.header(&headers::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap()[0].as_str(),
            "*"
        );
    }

    #[test]
    fn custom_cors_middleware() {
        let mut app = app();
        app.middleware(
            CorsMiddleware::new()
                .allow_origin(Origin::from(ALLOW_ORIGIN))
                .allow_credentials(false)
                .allow_methods(ALLOW_METHODS.parse::<HeaderValue>().unwrap())
                .expose_headers(EXPOSE_HEADER.parse::<HeaderValue>().unwrap()),
        );

        let mut server = make_server(app).unwrap();
        let res = server.simulate(request()).unwrap();

        assert_eq!(res.status(), 200);
        assert_eq!(
            res.header(&headers::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap()[0].as_str(),
            ALLOW_ORIGIN
        );
    }

    #[test]
    fn credentials_true() {
        let mut app = app();
        app.middleware(CorsMiddleware::new().allow_credentials(true));

        let mut server = make_server(app).unwrap();
        let res = server.simulate(request()).unwrap();

        assert_eq!(res.status(), 200);
        assert_eq!(
            res.header(&headers::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap()[0]
                .as_str(),
            "true"
        );
    }

    #[test]
    fn set_allow_origin_list() {
        let mut app = app();
        let origins = vec![ALLOW_ORIGIN, "foo.com", "bar.com"];
        app.middleware(CorsMiddleware::new().allow_origin(origins.clone()));
        let mut server = make_server(app).unwrap();

        for origin in origins {
            let mut request = http_types::Request::new(http_types::Method::Get, endpoint_url());
            request
                .insert_header(http_types::headers::ORIGIN, origin)
                .unwrap();

            let res = server.simulate(request).unwrap();

            assert_eq!(res.status(), 200);
            assert_eq!(
                res.header(&headers::ACCESS_CONTROL_ALLOW_ORIGIN),
                Some(&vec![origin.parse().unwrap()])
            );
        }
    }

    #[test]
    fn not_set_origin_header() {
        let mut app = app();
        app.middleware(CorsMiddleware::new().allow_origin(ALLOW_ORIGIN));

        let request = http_types::Request::new(http_types::Method::Get, endpoint_url());

        let mut server = make_server(app).unwrap();
        let res = server.simulate(request).unwrap();

        assert_eq!(res.status(), 200);
    }

    #[test]
    fn unauthorized_origin() {
        let mut app = app();
        app.middleware(CorsMiddleware::new().allow_origin(ALLOW_ORIGIN));

        let mut request = http_types::Request::new(http_types::Method::Get, endpoint_url());
        request
            .insert_header(http_types::headers::ORIGIN, "unauthorize-origin.net")
            .unwrap();

        let mut server = make_server(app).unwrap();
        let res = server.simulate(request).unwrap();

        assert_eq!(res.status(), 401);
    }
}
