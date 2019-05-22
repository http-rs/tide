use futures::future::BoxFuture;
use futures::prelude::*;
use http::{header::HeaderValue, HeaderMap, Method, Response, StatusCode};
use http_service::Body;
use tide::{
    middleware::{Middleware, Next},
    Context,
};

/// A blanket CORS middleware. It's customizable, but currently,
/// it's a simple blanket impl for the route tree than dynamic.
///
/// # Examples
///
/// ```rust
/// use surf::middlewares;
/// use http::header::HeaderValue;
///
/// let mut app = tide::App::new(());
/// app.middleware(middlewares::cors::CorsBlanket::new()
///      .origin(HeaderValue::from_str("https://surf-with-the-tide").unwrap())
///      .max_age(HeaderValue::from_str("600").unwrap()));
/// ```
///
#[derive(Debug, Clone)]
pub struct CorsBlanket {
    max_age: HeaderValue,
    methods: HeaderValue,
    origin: HeaderValue,
    headers: HeaderValue,
}

impl Default for CorsBlanket {
    fn default() -> Self {
        Self {
            max_age: HeaderValue::from_static(DEFAULT_MAX_AGE),
            methods: HeaderValue::from_static(DEFAULT_METHODS),
            origin: HeaderValue::from_static(STAR),
            headers: HeaderValue::from_static(STAR),
        }
    }
}

pub const DEFAULT_MAX_AGE: &str = "86400";
pub const DEFAULT_METHODS: &str = "GET, POST, OPTIONS";
pub const STAR: &str = "*";

impl CorsBlanket {
    pub fn new() -> Self {
        CorsBlanket::default()
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for CorsBlanket {
    fn handle<'a>(
        &'a self,
        ctx: Context<Data>,
        next: Next<'a, Data>,
    ) -> BoxFuture<'a, tide::Response> {
        use http::header;
        FutureExt::boxed(async move {
            if ctx.method() == Method::OPTIONS {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, self.origin.clone())
                    .header(header::ACCESS_CONTROL_ALLOW_METHODS, self.methods.clone())
                    .header(header::ACCESS_CONTROL_ALLOW_HEADERS, self.headers.clone())
                    .header(header::ACCESS_CONTROL_MAX_AGE, self.max_age.clone())
                    .body(Body::empty())
                    .unwrap();
            }
            let mut res = next.run(ctx).await;
            let headers: &mut HeaderMap = res.headers_mut();
            headers
                .entry(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap()
                .or_insert(self.origin.clone());
            res
        })
    }
}
