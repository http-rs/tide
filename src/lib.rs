// #![warn(missing_docs)]
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![allow(clippy::mutex_atomic, clippy::module_inception)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]
#![cfg_attr(feature = "docs", feature(doc_cfg))]

//! # Serve the web
//!
//! Tide is a friendly HTTP server built for casual Rustaceans and veterans alike. It's completely
//! modular, and built directly for `async/await`. Whether it's a quick webhook, or an L7 load
//! balancer, Tide will make it work.
//!
//! # Examples
//!
//! __hello world__
//! ```no_run
//! # use futures::executor::block_on;
//! # fn main() -> Result<(), std::io::Error> { block_on(async {
//! #
//! let mut app = tide::new();
//! app.at("/").get(|_| async move { "Hello, world!" });
//! app.listen("127.0.0.1:8080").await?;
//! #
//! # Ok(()) }) }
//! ````
//!
//! __echo server__
//! ```no_run
//! # use futures::executor::block_on;
//! # fn main() -> Result<(), std::io::Error> { block_on(async {
//! #
//! let mut app = tide::new();
//! app.at("/").get(|req| async move { req });
//! app.listen("127.0.0.1:8080").await?;
//! #
//! # Ok(()) }) }
//! ````
//!
//! __send and receive json__
//! _note: this example doesn't compile yet because we still need to work on
//! our error handling. Replace `?` with `.unwrap()` if you want to make this
//! compile_
//! ```ignore
//! # use futures::executor::block_on;
//! # fn main() -> Result<(), std::io::Error> { block_on(async {
//! #
//! #[derive(Debug, Deserialize, Serialize)]
//! struct Counter { count: usize }
//!
//! let mut app = tide::new();
//! app.at("/").get(|mut req: tide::Request<()>| async move {
//!    let mut counter: Counter = req.body_json().await?;
//!    println!("count is {}", counter.count);
//!    counter.count += 1;
//!    tide::Response::new(200).body_json(&counter)?
//! });
//! app.listen("127.0.0.1:8080").await?;
//! #
//! # Ok(()) }) }
//! ```
//!
//! # Stability
//!
//! It's still early in Tide's development cycle. While the general shape of Tide might have
//! roughly established, the exact traits and function paramaters may change between versions. In
//! practice this means that building your core business on Tide is probably not a wise idea...
//! yet.
//!
//! However we *are* committed to closely following semver, and documenting any and all breaking
//! changes we make. Also as time goes on you may find that fewer and fewer changes occur, until we
//! eventually remove this notice entirely.
//!
//! The goal of Tide is to build a premier HTTP experience for Async Rust. We have a long journey
//! ahead of us. But we're excited you're here with us!

mod endpoint;
mod error;
pub mod middleware;
mod redirect;
mod request;
mod response;
mod router;
mod utils;

pub mod prelude;
pub mod server;

pub use endpoint::Endpoint;
pub use error::{Error, Result, ResultExt};
pub use redirect::redirect;
pub use request::Request;

#[doc(inline)]
pub use middleware::{Middleware, Next};
#[doc(inline)]
pub use response::{IntoResponse, Response};
#[doc(inline)]
pub use server::{Route, Server};

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
