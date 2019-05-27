//! Cors middleware

use futures::future::BoxFuture;
use futures::prelude::*;
use http::header::HeaderValue;
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
///use tide::middleware::CorsMiddleware;
///
///CorsMiddleware::new()
///    .allow_origin(HeaderValue::from_static("*"))
///    .allow_methods(HeaderValue::from_static("GET, POST, OPTION"))
///    .allow_credentials(false)
///    .echo_back_origin(true);
/// ```
#[derive(Clone, Debug, Hash)]
pub struct CorsMiddleware {
    allow_credentials: Option<HeaderValue>,
    allow_headers: HeaderValue,
    allow_methods: HeaderValue,
    allow_origin: HeaderValue,
    expose_headers: Option<HeaderValue>,
    echo_back_origin: bool,
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
            allow_headers: HeaderValue::from_static(WILDCARD),
            allow_methods: HeaderValue::from_static(DEFAULT_METHODS),
            allow_origin: HeaderValue::from_static(WILDCARD),
            expose_headers: None,
            echo_back_origin: false,
            max_age: HeaderValue::from_static(DEFAULT_MAX_AGE),
        }
    }

    /// Set allow_credentials and return new CorsMiddleware
    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.allow_credentials = Some(HeaderValue::from_str(&allow_credentials.to_string()).unwrap());
        self
    }

    /// Set allow_headers and return new CorsMiddleware
    pub fn allow_headers(mut self, headers: impl Into<HeaderValue>) -> Self {
        self.allow_headers = headers.into();
        self
    }

    /// Set max_age and return new CorsMiddleware
    pub fn max_age(mut self, max_age: impl Into<HeaderValue>) -> Self {
        self.max_age = max_age.into();
        self
    }

    /// Set allow_methods and return new CorsMiddleware
    pub fn allow_methods(mut self, methods: impl Into<HeaderValue>) -> Self {
        self.allow_methods = methods.into();
        self
    }

    /// Set allow_origin and return new CorsMiddleware
    pub fn allow_origin(mut self, origin: impl Into<HeaderValue>) -> Self {
        self.allow_origin = origin.into();
        self
    }

    /// Set expose_headers and return new CorsMiddleware
    pub fn expose_headers(mut self, headers: impl Into<HeaderValue>) -> Self {
        self.expose_headers = Some(headers.into());
        self
    }

    /// Set echo_back_origin and return new CorsMiddleware
    pub fn echo_back_origin(mut self, echo_back_origin: bool) -> Self {
        self.echo_back_origin = echo_back_origin;
        self
    }
}

use http::{header, Method, StatusCode};
use http_service::Body;
impl<State: Send + Sync + 'static> Middleware<State> for CorsMiddleware {
    fn handle<'a>(&'a self, cx: Context<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        FutureExt::boxed(async move {
            // Return results immediately upon preflight request
            if cx.method() == Method::OPTIONS {
                let mut response = http::Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        self.allow_origin.clone(),
                    )
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

                if let Some(allow_credentials) = self.allow_credentials.clone(){
                    response.headers_mut().append(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, allow_credentials);
                }

                return response
            }

            let origin = if self.echo_back_origin {
                cx.headers()
                    .get("origin")
                    .unwrap_or(&HeaderValue::from_static(WILDCARD))
                    .clone()
            } else {
                self.allow_origin.clone()
            };

            let mut response = next.run(cx).await;
            let headers = response.headers_mut();

            headers.append(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);

            if let Some(expose_headers) = self.expose_headers.clone(){
                headers.append(header::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers);
            }

            if let Some(allow_credentials) = self.allow_credentials.clone(){
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
                .allow_origin(HeaderValue::from_static(ALLOW_ORIGIN))
                .allow_methods(HeaderValue::from_static(ALLOW_METHODS))
                .expose_headers(HeaderValue::from_static(EXPOSE_HEADER)),
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
                .allow_origin(HeaderValue::from_static(ALLOW_ORIGIN))
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
        assert_eq!(
            res.headers().get("access-control-expose-headers").unwrap(),
            EXPOSE_HEADER
        );
        assert_eq!(
            res.headers()
                .get("access-control-allow-credentials")
                .unwrap(),
            "false"
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
    fn echo_back_option() {
        let mut app = app();
        app.middleware(CorsMiddleware::new().echo_back_origin(true));

        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get(ENDPOINT)
            .header(http::header::ORIGIN, "foo")
            .method(http::method::Method::GET)
            .body(Body::empty())
            .unwrap();

        let res = server.simulate(req).unwrap();
        println!("{:?}", res.headers());

        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers().get("access-control-allow-origin").unwrap(),
            "foo"
        );
    }
}
