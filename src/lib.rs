//! # Serve the web
//!
//! Tide is a friendly HTTP server built for casual Rustaceans and veterans alike. It's completely
//! modular, and built directly for `async/await`. Whether it's a quick webhook, or an L7 load
//! balancer, Tide will make it work.
//!
//! # Features
//!
//! - __Fast:__ Written in Rust, and built on Futures, Tide is incredibly efficient.
//! - __Friendly:__ With thorough documentation, and a complete API, Tide helps cover your every
//!     need.
//! - __Minimal:__ With only a few concepts to learn, Tide is easy to pick up and become productive
//!     with.
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
//! ```
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
//! ```no_run
//! # use futures::executor::block_on;
//! # fn main() -> Result<(), std::io::Error> { block_on(async {
//! #
//! #[derive(Debug, serde::Deserialize, serde::Serialize)]
//! struct Counter { count: usize }
//!
//! let mut app = tide::new();
//! app.at("/").get(|mut req: tide::Request<()>| async move {
//!    let mut counter: Counter = req.body_json().await.unwrap();
//!    println!("count is {}", counter.count);
//!    counter.count += 1;
//!    tide::Response::new(http_types::StatusCode::Ok).body_json(&counter).unwrap()
//! });
//! app.listen("127.0.0.1:8080").await?;
//! #
//! # Ok(()) }) }
//! ```
//!
//! # Concepts
//!
//! ## Request-Response
//!
//! Each Tide endpoint takes a [`Request`] and returns a [`Response`]. Because async functions
//! allow us to wait without blocking, this makes Tide feel similar to synchronous servers. Except
//! it's incredibly efficient.
//!
//! ```txt
//! async fn endpoint(req: Request) -> Result<Response>;
//! ```
//!
//! ## Middleware
//!
//! Middleware wrap each request and response pair, allowing code to be run before the endpoint,
//! and after each endpoint. Additionally each handler can choose to never yield to the endpoint
//! and abort early. This is useful for e.g. authentication middleware. Tide's middleware works
//! like a stack. A simplified example of the logger middleware is something like this:
//!
//! ```ignore
//! async fn log(req: Request, next: Next) -> Result<Response> {
//!     println!("Incoming request from {} on url {}", req.peer_addr(), req.url());
//!     let res = next().await?;
//!     println!("Outgoing response with status {}", res.status());
//!     res
//! }
//! ```
//!
//! As a new request comes in, we perform some logic. Then we yield to the next
//! middleware (or endpoint, we don't know when we yield to `next`), and once that's
//! done, we return the Response. We can decide to not yield to `next` at any stage,
//! and abort early. This can then be used in applications using the [`Server::middleware`]
//! method.
//!
//! ## State
//!
//! Middleware often needs to share values with the endpoint. This is done through "local state".
//! Local state is built using a typemap that's available through [`Request::local`].
//!
//! Global state is used when a complete application needs access to a particular
//! value. Examples of this include: database connections, websocket connections, or
//! network-enabled config. Every `Request<State>` has an inner value that must
//! implement `Send + Sync + Clone`, and can thus freely be shared between requests.
//!
//! By default `tide::new` will use `()` as the shared state. But if you want to
//! create a new app with shared state you can use the [`with_state`] function.
//!
//! ## Extension Traits
//!
//! Sometimes having global and local context can require a bit of setup. There are
//! cases where it'd be nice if things were a little easier. This is why Tide
//! encourages people to write _extension traits_.
//!
//! By using an _extension trait_ you can extend [`Request`] or [`Response`] with more
//! functionality. For example, an authentication package could implement a `user` method on
//! `Request`, to access the authenticated user provided by middleware.
//!
//! Extension traits are written by defining a trait + trait impl for the struct that's being
//! extended:
//!
//! ```no_run
//! # use tide::Request;
//!
//! pub trait RequestExt {
//!     fn bark(&self) -> String;
//! }
//!
//! impl<State> RequestExt for Request<State> {
//!     fn bark(&self) -> String {
//!         "woof".to_string()
//!     }
//! }
//! ```
//!
//! Tide apps will then have access to the `bark` method on `Request`:
//!
//! ```no_run
//! # use tide::Request;
//! #
//! # pub trait RequestExt {
//! #     fn bark(&self) -> String;
//! # }
//!
//! # impl<State> RequestExt for Request<State> {
//! #     fn bark(&self) -> String {
//! #         "woof".to_string()
//! #     }
//! # }
//!
//! #[async_std::main]
//! async fn main() -> Result<(), std::io::Error> {
//!     let mut app = tide::new();
//!     app.at("/").get(|req: Request<()>| async move { req.bark() });
//!     app.listen("127.0.0.1:8080").await
//! }
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
//! eventually remove this notice entirely. The goal of Tide is to build a premier HTTP experience
//! for Async Rust. We have a long journey ahead of us. But we're excited you're here with us!

#![cfg_attr(feature = "docs", feature(doc_cfg))]
// #![warn(missing_docs)]
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

mod endpoint;
pub mod middleware;
mod redirect;
mod request;
mod response;
mod router;
mod utils;

pub mod prelude;
pub mod server;

pub use endpoint::Endpoint;
pub use redirect::redirect;
pub use request::Request;

#[doc(inline)]
pub use http_types::{Error, Result, Status};

#[doc(inline)]
pub use middleware::{Middleware, Next};
#[doc(inline)]
pub use response::{IntoResponse, Response};
#[doc(inline)]
pub use server::{Route, Server};

#[doc(inline)]
pub use http_types as http;

/// Create a new Tide server.
///
/// # Examples
///
/// ```no_run
/// # use futures::executor::block_on;
/// # fn main() -> Result<(), std::io::Error> { block_on(async {
/// #
/// let mut app = tide::new();
/// app.at("/").get(|_| async move { "Hello, world!" });
/// app.listen("127.0.0.1:8080").await?;
/// #
/// # Ok(()) }) }
/// ```
pub fn new() -> server::Server<()> {
    Server::new()
}

/// Create a new Tide server with shared global state.
///
/// Global state is useful for storing items
///
/// # Examples
///
/// ```no_run
/// # use futures::executor::block_on;
/// # fn main() -> Result<(), std::io::Error> { block_on(async {
/// #
/// use tide::Request;
///
/// /// The shared application state.
/// struct State {
///     name: String,
/// }
///
/// // Define a new instance of the state.
/// let state = State {
///     name: "Nori".to_string()
/// };
///
/// // Initialize the application with state.
/// let mut app = tide::with_state(state);
/// app.at("/").get(|req: Request<State>| async move {
///     format!("Hello, {}!", &req.state().name)
/// });
/// app.listen("127.0.0.1:8080").await?;
/// #
/// # Ok(()) }) }
/// ```
pub fn with_state<State>(state: State) -> server::Server<State>
where
    State: Send + Sync + 'static,
{
    Server::with_state(state)
}
