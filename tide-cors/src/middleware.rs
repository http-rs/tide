//! Cors middleware

use futures::future::BoxFuture;
use futures::prelude::*;
use http::header::HeaderValue;
use http::{header, Method, StatusCode};
use http_service::Body;
use tide_core::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// Middleware for CORS
///
/// # Example
///
/// ```rust
///use http::header::HeaderValue;
///use tide::middleware::{AllowOrigin, CorsMiddleware};
///
///CorsMiddleware::new()
///    .allow_origin(AllowOrigin::from("*"))
///    .allow_methods(HeaderValue::from_static("GET, POST, OPTIONS"))
///    .allow_credentials(false);
/// ```
#[derive(Clone, Debug, Hash)]
pub struct CorsMiddleware {
    allow_credentials: Option<HeaderValue>,
    allow_headers: HeaderValue,
    allow_methods: HeaderValue,
    allow_origin: AllowOrigin,
    expose_headers: Option<HeaderValue>,
    max_age: HeaderValue,
}

/// allow_origin enum
#[derive(Clone, Debug, Hash, PartialEq)]
pub enum AllowOrigin {
    /// Wildcard. Accept all origin requests
    Any,
    /// Set one allow_origin
    Exact(String),
    /// Set some allow_origin
    List(Vec<String>),
}

impl From<String> for AllowOrigin {
    fn from(s: String) -> Self {
        if s == "*" {
            return AllowOrigin::Any;
        }
        AllowOrigin::Exact(s)
    }
}

impl From<&str> for AllowOrigin {
    fn from(s: &str) -> Self {
        AllowOrigin::from(s.to_string())
    }
}

impl From<Vec<String>> for AllowOrigin {
    fn from(list: Vec<String>) -> Self {
        if list.len() == 1 {
            return Self::from(list[0].clone());
        }
        return AllowOrigin::List(list);
    }
}

impl From<Vec<&str>> for AllowOrigin {
    fn from(list: Vec<&str>) -> Self {
        AllowOrigin::from(list.iter().map(|s| s.to_string()).collect::<Vec<String>>())
    }
}

pub const DEFAULT_MAX_AGE: &str = "86400";
pub const DEFAULT_METHODS: &str = "GET, POST, OPTIONS";
pub const WILDCARD: &str = "*";

impl CorsMiddleware {
    /// Creates a new Cors middleware.
    pub fn new() -> Self {
        Self {
            allow_credentials: None,
            allow_headers: HeaderValue::from_static(WILDCARD),
            allow_methods: HeaderValue::from_static(DEFAULT_METHODS),
            allow_origin: AllowOrigin::Any,
            expose_headers: None,
            max_age: HeaderValue::from_static(DEFAULT_MAX_AGE),
        }
    }

    /// Set allow_credentials and return new CorsMiddleware
    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.allow_credentials = match HeaderValue::from_str(&allow_credentials.to_string()) {
            Ok(header) => Some(header),
            Err(_) => None,
        };
        self
    }

    /// Set allow_headers and return new CorsMiddleware
    pub fn allow_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.allow_headers = headers.into();
        self
    }

    /// Set max_age and return new CorsMiddleware
    pub fn max_age<T: Into<HeaderValue>>(mut self, max_age: T) -> Self {
        self.max_age = max_age.into();
        self
    }

    /// Set allow_methods and return new CorsMiddleware
    pub fn allow_methods<T: Into<HeaderValue>>(mut self, methods: T) -> Self {
        self.allow_methods = methods.into();
        self
    }

    /// Set allow_origin and return new CorsMiddleware
    pub fn allow_origin<T: Into<AllowOrigin>>(mut self, origin: T) -> Self {
        self.allow_origin = origin.into();
        self
    }

    /// Set expose_headers and return new CorsMiddleware
    pub fn expose_headers<T: Into<HeaderValue>>(mut self, headers: T) -> Self {
        self.expose_headers = Some(headers.into());
        self
    }

    fn build_preflight_response(&self, origin: &HeaderValue) -> http::response::Response<Body> {
        let mut response = http::Response::builder()
            .status(StatusCode::OK)
            .header::<_, HeaderValue>(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone())
            .header(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                self.allow_methods.clone(),
            )
            .header(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                self.allow_headers.clone(),
            )
            .header(header::ACCESS_CONTROL_MAX_AGE, self.max_age.clone())
            .body(Body::empty())
            .unwrap();

        if let Some(allow_credentials) = self.allow_credentials.clone() {
            response
                .headers_mut()
                .append(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials);
        }

        if let Some(expose_headers) = self.expose_headers.clone() {
            response
                .headers_mut()
                .append(header::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers);
        }

        response
    }

    /// Look at origin of request and determine allow_origin
    fn response_origin<T: Into<HeaderValue>>(&self, origin: T) -> Option<HeaderValue> {
        let origin = origin.into();
        if !self.is_valid_origin(origin.clone()) {
            return None;
        }

        match self.allow_origin {
            AllowOrigin::Any => Some(HeaderValue::from_static(WILDCARD)),
            _ => Some(origin.into()),
        }
    }

    /// Determine if origin is appropriate
    fn is_valid_origin<T: Into<HeaderValue>>(&self, origin: T) -> bool {
        let origin = match origin.into().to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return false,
        };

        match &self.allow_origin {
            AllowOrigin::Any => true,
            AllowOrigin::Exact(s) => s == &origin,
            AllowOrigin::List(list) => list.contains(&origin),
        }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for CorsMiddleware {
    fn handle<'a>(&'a self, cx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        FutureExt::boxed(async move {
            let origin = if let Some(origin) = cx.request().headers().get(header::ORIGIN) {
                origin.clone()
            } else {
                return http::Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::empty())
                    .unwrap();
            };

            if !self.is_valid_origin(&origin) {
                return http::Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .unwrap();
            }

            // Return results immediately upon preflight request
            if cx.method() == Method::OPTIONS {
                return self.build_preflight_response(&origin);
            }

            let mut response = next.run(cx).await;
            let headers = response.headers_mut();

            headers.append(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                self.response_origin(origin).unwrap(),
            );

            if let Some(allow_credentials) = self.allow_credentials.clone() {
                headers.append(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials);
            }
            response
        })
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use http::header::HeaderValue;
    use http_service::Body;
    use http_service_mock::make_server;

    const ALLOW_ORIGIN: &str = "example.com";
    const ALLOW_METHODS: &str = "GET, POST, OPTIONS, DELETE";
    const EXPOSE_HEADER: &str = "X-My-Custom-Header";

    const ENDPOINT: &str = "/cors";

    fn app() -> tide_core::App<()> {
        let mut app = tide_core::App::new();
        app.at(ENDPOINT).get(async move |_| "Hello World");

        app
    }

    fn request() -> http::Request<http_service::Body> {
        http::Request::get(ENDPOINT)
            .header(http::header::ORIGIN, ALLOW_ORIGIN)
            .method(http::method::Method::GET)
            .body(Body::empty())
            .unwrap()
    }

    #[test]
    fn preflight_request() {
        let mut app = app();
        app.middleware(
            CorsMiddleware::new()
                .allow_origin(AllowOrigin::from(ALLOW_ORIGIN))
                .allow_methods(HeaderValue::from_static(ALLOW_METHODS))
                .expose_headers(HeaderValue::from_static(EXPOSE_HEADER))
                .allow_credentials(true),
        );

        let mut server = make_server(app.into_http_service()).unwrap();

        let req = http::Request::get(ENDPOINT)
            .header(http::header::ORIGIN, ALLOW_ORIGIN)
            .method(http::method::Method::OPTIONS)
            .body(Body::empty())
            .unwrap();

        let res = server.simulate(req).unwrap();

        assert_eq!(res.status(), 200);

        assert_eq!(
            res.headers().get("access-control-allow-origin").unwrap(),
            ALLOW_ORIGIN
        );
        assert_eq!(
            res.headers().get("access-control-allow-methods").unwrap(),
            ALLOW_METHODS
        );
        assert_eq!(
            res.headers().get("access-control-allow-headers").unwrap(),
            WILDCARD
        );
        assert_eq!(
            res.headers().get("access-control-max-age").unwrap(),
            DEFAULT_MAX_AGE
        );

        assert_eq!(
            res.headers()
                .get("access-control-allow-credentials")
                .unwrap(),
            "true"
        );
    }
    #[test]
    fn default_cors_middleware() {
        let mut app = app();
        app.middleware(CorsMiddleware::new());

        let mut server = make_server(app.into_http_service()).unwrap();
        let res = server.simulate(request()).unwrap();

        assert_eq!(res.status(), 200);

        assert_eq!(
            res.headers().get("access-control-allow-origin").unwrap(),
            "*"
        );
    }

    #[test]
    fn custom_cors_middleware() {
        let mut app = app();
        app.middleware(
            CorsMiddleware::new()
                .allow_origin(AllowOrigin::from(ALLOW_ORIGIN))
                .allow_credentials(false)
                .allow_methods(HeaderValue::from_static(ALLOW_METHODS))
                .expose_headers(HeaderValue::from_static(EXPOSE_HEADER)),
        );

        let mut server = make_server(app.into_http_service()).unwrap();
        let res = server.simulate(request()).unwrap();

        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers().get("access-control-allow-origin").unwrap(),
            ALLOW_ORIGIN
        );
    }

    #[test]
    fn credentials_true() {
        let mut app = app();
        app.middleware(CorsMiddleware::new().allow_credentials(true));

        let mut server = make_server(app.into_http_service()).unwrap();
        let res = server.simulate(request()).unwrap();

        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers()
                .get("access-control-allow-credentials")
                .unwrap(),
            "true"
        );
    }
    #[test]
    fn set_allow_origin_list() {
        let mut app = app();
        let origins = vec![ALLOW_ORIGIN, "foo.com", "bar.com"];
        app.middleware(CorsMiddleware::new().allow_origin(origins.clone()));
        let mut server = make_server(app.into_http_service()).unwrap();

        for origin in origins {
            let request = http::Request::get(ENDPOINT)
                .header(http::header::ORIGIN, origin)
                .method(http::method::Method::GET)
                .body(Body::empty())
                .unwrap();

            let res = server.simulate(request).unwrap();

            assert_eq!(res.status(), 200);
            assert_eq!(
                res.headers().get("access-control-allow-origin").unwrap(),
                origin
            );
        }
    }

    #[test]
    fn not_set_origin_header() {
        let mut app = app();
        app.middleware(CorsMiddleware::new());

        let request = http::Request::get(ENDPOINT)
            .method(http::method::Method::GET)
            .body(Body::empty())
            .unwrap();

        let mut server = make_server(app.into_http_service()).unwrap();
        let res = server.simulate(request).unwrap();

        assert_eq!(res.status(), 400);
    }

    #[test]
    fn unauthorized_origin() {
        let mut app = app();
        app.middleware(CorsMiddleware::new().allow_origin(ALLOW_ORIGIN));

        let request = http::Request::get(ENDPOINT)
            .header(http::header::ORIGIN, "unauthorize-origin.net")
            .method(http::method::Method::GET)
            .body(Body::empty())
            .unwrap();

        let mut server = make_server(app.into_http_service()).unwrap();
        let res = server.simulate(request).unwrap();

        assert_eq!(res.status(), 401);
    }
}
