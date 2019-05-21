#![cfg_attr(any(feature = "nightly", test), feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![feature(async_await, existential_type)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
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

#[cfg(feature = "cookies")]
#[doc(inline)]
pub use tide_cookies as cookies;

#[doc(inline)]
pub use tide_core::{
    err_fmt,
    response,
    App,
    Context,
    Endpoint,
    EndpointResult,
    Error,
    Response,
    Route,
    Server,
    // TODO: export Body once it's in turn exported by tide_core
};

pub mod error {
    pub use tide_core::error::{
        EndpointResult, Error, ResponseExt, ResultDynErrExt, ResultExt, StringError,
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
