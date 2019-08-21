//! Crate that provides helpers and/or middlewares for Tide
//! related to structured logging with slog.

#![cfg_attr(docrs, feature(doc_cfg))]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

mod per_request_logger;
mod request_logger;
#[cfg(feature = "scope")]
mod set_slog_scope_logger;

pub use per_request_logger::PerRequestLogger;
pub use request_logger::RequestLogger;
#[cfg(feature = "scope")]
pub use set_slog_scope_logger::SetSlogScopeLogger;

use tide_core::Context;

/// An extension to [`Context`] that provides access to a per-request [`slog::Logger`]
pub trait ContextExt {
    /// Returns a [`slog::Logger`] scoped to this request.
    ///
    /// # Panics
    ///
    /// Will panic if no [`PerRequestLogger`] middleware has been used to setup the request scoped
    /// logger.
    fn logger(&self) -> &slog::Logger;
}

impl<State> ContextExt for Context<State> {
    fn logger(&self) -> &slog::Logger {
        self.extensions()
            .get::<slog::Logger>()
            .expect("PerRequestLogger must be used to populate request logger")
    }
}
