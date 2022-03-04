//! Server-Sent Events (SSE) types.
//!
//! # Errors
//!
//! Errors originating in the SSE handler will be logged. Errors originating
//! during the encoding of the SSE stream will be handled by the backend engine
//! the way any other IO error is handled.
//!
//! In the future we may introduce a better mechanism to handle errors that
//! originate outside of regular endpoints.
//!
//! # Examples
//!
//! ```no_run
//! # fn main() -> Result<(), std::io::Error> { async_std::task::block_on(async {
//! #
//! use tide::sse;
//!
//! let mut app = tide::new();
//! app.at("/sse").get(sse::endpoint(|_req, sender| async move {
//!     sender.send("fruit", "banana", None).await?;
//!     sender.send("fruit", "apple", None).await?;
//!     Ok(())
//! }));
//! app.listen("localhost:8080").await?;
//! # Ok(()) }) }
//! ```
mod endpoint;
mod sender;
mod upgrade;

pub use endpoint::{endpoint, SseEndpoint};
pub use sender::Sender;
pub use upgrade::upgrade;
