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
//!         let mut app = tide::Server::new();
//!         app.at("/").get(|_| async move { "Hello, world!" });
//!         app.listen("127.0.0.1:8000").await?;
//!         Ok(())
//!     })
//! }
//! ````

#[macro_use]
pub mod error;

mod context;
mod endpoint;
mod route;
mod router;
mod server;

pub mod cookies;
pub mod forms;
pub mod middleware;
pub mod querystring;
pub mod response;

#[doc(inline)]
pub use crate::{
    context::Context,
    endpoint::Endpoint,
    error::{EndpointResult, Error},
    response::Response,
    route::Route,
    server::{Server, Service},
};

#[doc(inline)]
pub use http;
