// #![warn(missing_docs)]
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![allow(clippy::mutex_atomic, clippy::module_inception)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

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
