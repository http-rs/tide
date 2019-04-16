#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(test, deny(warnings))]
#![allow(unused_variables)]
#![feature(futures_api, async_await, await_macro, existential_type)]

//!
//! Welcome to Tide.
//!
//! The [`App`](struct.App.html) docs are a good place to get started.
//!
//!

macro_rules! box_async {
    {$($t:tt)*} => {
        FutureObj::new(Box::new(async move { $($t)* }))
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
