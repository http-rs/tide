//! Event logging types.
//!
//! # Examples
//!
//! ```no_run
//! use tide::log;
//!
//! // `tide::log` requires starting a third-party logger such as `femme`. We may
//! // ship such a logger as part of Tide in the future.
//! femme::start(log::Level::Info.to_level_filter()).unwrap();
//!
//! log::info!("Hello cats");
//! log::debug!("{} wants tuna", "Nori");
//! log::error!("We're out of tuna!");
//! log::info!("{} are hungry", "cats", {
//!     cat_1: "Chashu",
//!     cat_2: "Nori",
//! });
//! ```

pub use kv_log_macro::{debug, error, info, log, trace, warn};
pub use kv_log_macro::{max_level, Level};

mod middleware;

pub use middleware::LogMiddleware;
