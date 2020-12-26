//! HTTP rate, size, and load limiting middleware.
mod load_shedder;

pub use load_shedder::LoadShedder;
