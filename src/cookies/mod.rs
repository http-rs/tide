//! HTTP cookies.

mod middleware;

pub(crate) use middleware::CookieData;
pub use middleware::CookiesMiddleware;
