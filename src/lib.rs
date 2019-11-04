// #![warn(missing_docs)]
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![allow(clippy::mutex_atomic, clippy::module_inception)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

//! # Serve the web
//!
//! Tide is a friendly HTTP server built for casual Rustaceans and veterans alike. It's completely
//! modular, and built directly for `async/await`. Whether it's a light webhook, or an L7 load
//! balancer, Tide will make it work.
//!
//! # Examples
//!
//! ```no_run
//! # use futures::executor::block_on;
//! # fn main() -> Result<(), std::io::Error> { block_on(async {
//! let mut app = tide::new();
//! app.at("/").get(|_| async move { "Hello, world!" });
//! app.listen("127.0.0.1:8000").await?;
//! # Ok(()) }) }
//! ````

mod endpoint;
mod request;
mod router;
mod utils;

pub mod server;

#[macro_use]
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod cookies;
#[doc(hidden)]
pub mod middleware;
#[doc(hidden)]
pub mod response;

pub use endpoint::Endpoint;
#[doc(inline)]
pub use error::Result;
pub use request::Request;
#[doc(inline)]
pub use response::{IntoResponse, Response};
#[doc(inline)]
pub use server::{Route, Server};

#[doc(hidden)]
pub use error::Error;

pub use http;

/// Create a new Tide server.
pub fn new() -> server::Server<()> {
    Server::new()
}

/// Create a new Tide server with shared global state.
pub fn with_state<State>(state: State) -> server::Server<State>
where
    State: Send + Sync + 'static,
{
    Server::with_state(state)
}
