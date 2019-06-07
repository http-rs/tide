//! Core types and traits from Tide

#![feature(async_await, existential_type)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
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

// Internal shared API for limited use across crates in our repo
pub mod internal;

pub use crate::{
    app::{App, Server},
    context::Context,
    endpoint::Endpoint,
    error::{EndpointResult, Error},
    response::Response,
    route::Route,
};
