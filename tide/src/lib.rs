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
extern crate tide_core;

#[cfg(feature = "cookies")]
#[doc(inline)]
pub use tide_cookies as cookies;

pub mod error;
pub mod forms;
pub mod middleware;
pub mod querystring;

#[doc(inline)]
pub use tide_core::{response, App, Context, Endpoint, Error, Response, Result, Route, Server};

pub use http;
