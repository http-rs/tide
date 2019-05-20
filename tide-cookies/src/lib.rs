#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(test, deny(warnings))]
#![feature(async_await)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

#[macro_use]
extern crate tide_core;

mod data;
mod middleware;

pub use self::data::ContextExt;
pub use self::middleware::CookiesMiddleware;
