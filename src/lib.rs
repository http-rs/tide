#![cfg_attr(any(feature = "nightly", test), feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![feature(async_await, existential_type)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

//!
//! Welcome to Tide.
//!
//! The [`App`](struct.App.html) docs are a good place to get started.
//!
//!

#[cfg(test)]
#[doc(include = "../README.md")]
const _README: () = ();

pub use http;

mod app;

pub use app::{App, Server};

#[cfg(feature = "cookies")]
#[doc(inline)]
pub use tide_cookies as cookies;

#[doc(inline)]
pub use tide_core::{err_fmt, response, Body, Context, Endpoint, EndpointResult, Error, Response};

pub mod error {
    pub use tide_core::error::{
        Error, ResponseExt, ResultDynErrExt, ResultExt, StringError,
    };
}

pub use tide_forms as forms;
pub use tide_querystring as querystring;

pub mod middleware {
    // Core
    pub use tide_core::middleware::{Middleware, Next};

    // Exports from tide repo.
    pub use tide_headers::DefaultHeaders;
    pub use tide_log::RequestLogger;

    #[cfg(feature = "cookies")]
    pub use tide_cookies::CookiesMiddleware;
}
