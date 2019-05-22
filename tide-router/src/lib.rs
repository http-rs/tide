//! Crate that provides helpers and/or middlewares for Tide
//! related to routing.

#![feature(async_await)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

mod route;
mod router;

pub use route::Route;
pub use router::{Router, Selection};
