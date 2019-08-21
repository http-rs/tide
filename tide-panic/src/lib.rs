//! Advanced panic support for Tide applications.
//!
//! These middleware should not generally be necessary, they are provided for situations in which
//! Tide's default panic handling is not usable by your application. Before using these you should
//! have a good understanding of how the different components involved in [`std::panic`] works.

#![feature(doc_cfg)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations,
    missing_docs
)]

mod catch_unwind;

pub use crate::catch_unwind::CatchUnwind;
