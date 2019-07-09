//! This crate provides cors-related middleware for Tide.
//!
//! ## Examples
//!
//! ```rust,no_run
//! #![feature(async_await, async_closure)]
//!
//! use http::header::HeaderValue;
//! use tide_cors::CorsMiddleware;
//!
//! fn main() {
//!     let mut app = tide::App::new();
//!
//!     app.middleware(
//!         CorsMiddleware::new()
//!             .allow_origin(HeaderValue::from_static("*"))
//!             .allow_methods(HeaderValue::from_static("GET, POST, OPTIONS")),
//!     );
//!
//!     app.at("/").get(async move |_| "Hello, world!");
//!
//!     app.run("127.0.0.1:8000").unwrap();
//! }
//! ```
//! You can test this by running the following in your browser:
//!
//! ```console
//! $ fetch("http://127.0.0.1:8000")
//! ```
//!
//! You will probably get a browser alert when running without cors middleware.

#![feature(async_await, async_closure)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
)]

mod middleware;

pub use self::middleware::CorsMiddleware;
