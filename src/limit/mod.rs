//! HTTP rate, size, and load limiting middleware.
#[cfg(feature = "sessions")]
mod load_shedder;

#[cfg(feature = "sessions")]
pub use load_shedder::LoadShedder;
