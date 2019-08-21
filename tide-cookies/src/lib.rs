//! Crate that provides helpers and/or middlewares for Tide
//! related to cookies.

#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
)]

mod data;
mod middleware;

pub use self::data::ContextExt;
pub use self::middleware::CookiesMiddleware;
