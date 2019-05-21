#![feature(async_await)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

mod data;
mod middleware;

pub use self::data::ContextExt;
pub use self::middleware::CookiesMiddleware;
