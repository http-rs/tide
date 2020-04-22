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
//!     sender.send("fruit", "banana").await;
//!     sender.send("fruit", "apple").await;
//!     Ok(())
//! }));
//! app.listen("localhost:8080").await?;
//! # Ok(()) }) }
//! ```

mod endpoint;
mod upgrade;

pub use endpoint::{endpoint, SseEndpoint};
pub use upgrade::upgrade;

/// An SSE message sender.
#[derive(Debug)]
pub struct Sender {
    sender: async_sse::Sender,
}

impl Sender {
    /// Create a new instance of `Sender`.
    fn new(sender: async_sse::Sender) -> Self {
        Self { sender }
    }

    /// Send data from the SSE channel.
    ///
    /// Each message constists of a "name" and "data".
    pub async fn send(&self, name: &str, data: impl AsRef<[u8]>) {
        self.sender.send(name, data.as_ref(), None).await;
    }
}
