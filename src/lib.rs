#![cfg_attr(any(feature = "nightly", test), feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![feature(async_await, existential_type)]
#![allow(unused_variables)]
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
pub use tide_core::{
    response, App, Context, Endpoint, EndpointResult, Error, Response, Route, Server,
};

pub use http;
