#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(any(feature = "nightly", test), feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(test, deny(warnings))]
#![feature(async_await, existential_type)]
#![allow(unused_variables)]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]
// TODO: Remove this after clippy bug due to async await is resolved.
// ISSUE: https://github.com/rust-lang/rust-clippy/issues/3988
#![allow(clippy::needless_lifetimes)]

//!
//! Welcome to Tide.
//!
//! The [`App`](struct.App.html) docs are a good place to get started.
//!
//!

#[cfg(test)]
#[doc(include = "../../README.md")]
const _README: () = ();

macro_rules! box_async {
    {$($t:tt)*} => {
        ::futures::future::FutureExt::boxed(async move { $($t)* })
    };
}

#[macro_use]
pub mod error;

pub mod cookies;
pub mod forms;
pub mod middleware;
pub mod querystring;

pub use tide_core::response;

#[doc(inline)]
pub use tide_core::{App, Context, Endpoint, EndpointResult, Error, Response, Route, Server};

pub use http;
