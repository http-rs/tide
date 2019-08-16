//! Core types and traits from Tide

#![feature(async_await)]
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

mod context;
mod endpoint;

pub mod error;
pub mod middleware;
pub mod response;

// Internal shared API for limited use across crates in our repo
pub mod internal;

pub use crate::{
    context::Context,
    endpoint::{Endpoint, EndpointResult},
    error::Error,
    response::{Body, Response},
};
