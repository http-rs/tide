//! Event logging types.
//!
//! # Examples
//!
//! ```no_run
//! use tide::log;
//!
//! log::start();
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

#[cfg(feature = "logger")]
pub use femme::LevelFilter;

pub use middleware::LogMiddleware;

/// Start logging.
#[cfg(feature = "logger")]
pub fn start() {
    femme::start();
    crate::log::info!("Logger started", { level: "Info" });
}

/// Start logging with a log level.
#[cfg(feature = "logger")]
pub fn with_level(level: LevelFilter) {
    femme::with_level(level);
    crate::log::info!("Logger started", { level: format!("{}", level) });
}
