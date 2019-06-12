//! This crate provides cors-related middleware for Tide.
//!
//! ## Examples
//!
//! Examples are in the `/examples` folder of this crate.
//!
//! ```rust,no_run
//! #![feature(async_await)]
//!
//! use http::header::HeaderValue;
//! use tide::middleware::{CorsMiddleware, AllowOrigin};
//!
//! fn main() {
//!     let mut app = tide::App::new();
//!
//!     app.middleware(
//!         CorsMiddleware::new()
//!             .allow_origin(AllowOrigin::from("*"))
//!             .allow_methods(HeaderValue::from_static("GET, POST, OPTIONS")),
//!     );
//!
//!     app.at("/").get(async move |_| "Hello, world!");
//!
//!     app.run("127.0.0.1:8000").unwrap();
//! }
//! ```
//!
//! __Simple Example__
//!
//! You can test the simple example by running `cargo run --example cors` while in this crate's directory, and then running this script in the browser console:
//!
//! ```console
//! $ fetch("http://127.0.0.1:8000")
//! ```
//!
//! You will probably get a browser alert when running without cors middleware.

#![feature(async_await)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
)]

mod middleware;

pub use self::middleware::{AllowOrigin, CorsMiddleware};
