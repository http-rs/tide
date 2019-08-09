//! Welcome to Tide.
//!
//! The [`App`](struct.App.html) docs are a good place to get started.

#![cfg_attr(any(feature = "nightly", test), feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![feature(async_await)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
)]

#[cfg(test)]
#[doc(include = "../README.md")]
const _README: () = ();

pub use http;

mod app;
mod router;

pub use app::{App, Server};

#[cfg(feature = "cookies")]
#[doc(inline)]
pub use tide_cookies as cookies;

#[cfg(feature = "cors")]
#[doc(inline)]
pub use tide_cors as cors;

#[doc(inline)]
pub use tide_core::{response, Body, Context, Endpoint, EndpointResult, Error, Response};

pub mod error {
    //! Error types re-exported from `tide-core`
    pub use tide_core::error::{Error, ResponseExt, ResultDynErrExt, ResultExt, StringError};
}

pub use tide_forms as forms;
pub use tide_querystring as querystring;

pub mod middleware {
    //! Module to export tide_core middleware

    // Core
    pub use tide_core::middleware::{Middleware, Next};

    // Exports from tide repo.
    pub use tide_headers::DefaultHeaders;
    pub use tide_log::RequestLogger;

    #[cfg(feature = "cors")]
    pub use tide_cors::{CorsMiddleware, CorsOrigin};

    #[cfg(feature = "cookies")]
    pub use tide_cookies::CookiesMiddleware;
}
