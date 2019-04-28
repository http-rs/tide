#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
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

macro_rules! box_async {
    {$($t:tt)*} => {
        ::futures::future::FutureExt::boxed(async move { $($t)* })
    };
}

#[macro_use]
pub mod error;

mod app;
mod context;
pub mod cookies;
mod endpoint;
pub mod forms;
pub mod middleware;
pub mod querystring;
pub mod response;
mod route;
mod router;

#[doc(inline)]
pub use crate::{
    app::{App, Server},
    context::Context,
    endpoint::Endpoint,
    error::{EndpointResult, Error},
    response::Response,
    route::Route,
};

pub use http;
