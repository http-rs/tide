#![feature(async_await, existential_type)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]
// TODO: Remove this after clippy bug due to async await is resolved.
// ISSUE: https://github.com/rust-lang/rust-clippy/issues/3988
#![allow(clippy::needless_lifetimes)]

mod app;
mod context;
mod endpoint;
pub mod error;
pub mod middleware;
pub mod response;
mod route;
mod router;

pub use crate::{
    app::{App, Server},
    context::Context,
    endpoint::Endpoint,
    error::{EndpointResult, Error},
    response::Response,
    route::Route,
};
