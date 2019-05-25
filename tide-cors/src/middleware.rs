//! Cors middleware

use futures::future::BoxFuture;
use futures::prelude::*;
use http::header::HeaderValue;
use tide_core::{
    middleware::{Middleware, Next},
    Context, Response,
};

/// Middleware for CORS
#[derive(Clone, Debug, Hash)]
pub struct CorsMiddleware {
    allow_credentials: HeaderValue,
    allow_headers: HeaderValue,
    allow_methods: HeaderValue,
    allow_origin: HeaderValue,
    expose_headers: HeaderValue,
    echo_back_origin: bool,
    max_age: HeaderValue,
}

pub const DEFAULT_MAX_AGE: &str = "86400";
pub const DEFAULT_METHODS: &str = "GET, POST, OPTIONS";
pub const EMPTY_HEADER: &str = "";
pub const WILDCARD: &str = "*";

impl CorsMiddleware {
    /// Creates a new Cors middleware.
    pub fn new() -> Self {
        Self {
            allow_credentials: HeaderValue::from_static("false"),
            allow_headers: HeaderValue::from_static(WILDCARD),
            allow_methods: HeaderValue::from_static(DEFAULT_METHODS),
            allow_origin: HeaderValue::from_static(WILDCARD),
            expose_headers: HeaderValue::from_static(EMPTY_HEADER),
            echo_back_origin: false,
            max_age: HeaderValue::from_static(DEFAULT_MAX_AGE),
        }
    }

    /// Set allow_credentials and return new CorsMiddleware
    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.allow_credentials = HeaderValue::from_str(&allow_credentials.to_string()).unwrap();
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
        self.expose_headers = headers.into();
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
                return http::Response::builder()
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
            headers.append(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                self.allow_headers.clone(),
            );
            headers.append(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                self.allow_methods.clone(),
            );
            headers.append(header::ACCESS_CONTROL_MAX_AGE, self.max_age.clone());
            headers.append(
                header::ACCESS_CONTROL_EXPOSE_HEADERS,
                self.expose_headers.clone(),
            );
            headers.append(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                self.allow_credentials.clone(),
            );

            response
        })
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
