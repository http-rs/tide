//! HTTP redirection endpoints.
//!
//! # Examples
//!
//! ```no_run
//! # use futures::executor::block_on;
//! # fn main() -> Result<(), std::io::Error> { block_on(async {
//! #
//! use tide::redirect;
//!
//! let mut app = tide::new();
//! app.at("/").get(|_| async move { Ok("meow") });
//! app.at("/nori").get(redirect::temporary("/"));
//! app.listen("127.0.0.1:8080").await?;
//! #
//! # Ok(()) }) }
//! ```
mod permanent;
mod temporary;

pub use permanent::{permanent, PermanentRedirect};
pub use temporary::{temporary, TemporaryRedirect};
