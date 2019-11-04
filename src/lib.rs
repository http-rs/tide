// #![warn(missing_docs)]
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![allow(clippy::mutex_atomic, clippy::module_inception)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

//! Tide is a friendly HTTP server framework.
//!
//! # Examples
//!
//! ```no_run
//! use futures::executor::block_on;
//!
//! fn main() -> Result<(), std::io::Error> {
//!     block_on(async {
//!         let mut app = tide::new();
//!         app.at("/").get(|_| async move { "Hello, world!" });
//!         app.listen("127.0.0.1:8000").await
//!     })
//! }
//! ````

#[macro_use]
pub mod error;

mod context;
mod endpoint;
mod route;
mod router;

pub mod server;

#[doc(hidden)]
pub mod cookies;
#[doc(hidden)]
pub mod forms;
#[doc(hidden)]
pub mod middleware;
#[doc(hidden)]
pub mod querystring;
#[doc(hidden)]
pub mod response;

#[doc(inline)]
pub use crate::{
    context::Request,
    endpoint::Endpoint,
    error::{EndpointResult, Error},
    response::Response,
    route::Route,
    server::Server,
};

pub use http;

/// Create a new Tide server.
pub fn new() -> server::Server<()> {
    Server::new()
}

/// Create a new Tide server with state
pub fn with_state<State>(state: State) -> server::Server<State>
where
    State: Send + Sync + 'static,
{
    Server::with_state(state)
}
