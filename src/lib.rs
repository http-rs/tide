//! Welcome to Tide.
//!
//! The [`Server`](struct.Server.html) docs are a good place to get started.
//!
//! # Examples
//! ```
//! # #![feature(async_await)]
//! #[runtime::main]
//! async fn main() -> Result<(), tide::Exception> {
//!     let mut app = tide::new();
//!     app.at("/").get(|_| async move { "Hello, world!" });
//!     app.bind("127.0.0.1:8000").await
//! }
//! ```

#![cfg_attr(any(feature = "nightly", test), feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![feature(async_await, existential_type)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
)]

#[cfg(test)]
#[doc(include = "../README.md")]
const _README: () = ();

#[doc(hidden)]
pub mod error;
pub mod middleware;
pub mod router;
pub mod server;

mod request;
mod response;

pub use http;
pub use mime;
pub use tide_core;
pub use url;

pub use request::Request;
pub use response::Response;

#[doc(inline)]
pub use server::Server;

#[doc(hidden)]
pub use tide_core::{Body, Context, Endpoint};

/// Catch-all error type.
pub type Exception = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Create a new Tide server.
pub fn new() -> Server<()> {
    Server::new()
}

/// Create a new Tide server with shared global state.
pub fn with_state<State: Send + Sync + 'static>(state: State) -> Server<State> {
    Server::with_state(state)
}
