//! Types that represent HTTP transports and binding

mod concurrent_listener;
mod parsed_listener;
mod tcp_listener;
mod to_listener;
#[cfg(unix)]
mod unix_listener;

use crate::utils::BoxFuture;
use crate::Server;
use async_std::io;

pub use concurrent_listener::ConcurrentListener;
pub use to_listener::ToListener;

pub(crate) use parsed_listener::ParsedListener;
pub(crate) use tcp_listener::TcpListener;
#[cfg(unix)]
pub(crate) use unix_listener::UnixListener;

/// The Listener trait represents an implementation of http transport
/// for a tide application. In order to provide a Listener to tide,
/// you will also need to implement at least one [`ToListener`](crate::listener::ToListener) that
/// outputs your Listener type.
pub trait Listener<State: 'static>:
    std::fmt::Debug + std::fmt::Display + Send + Sync + 'static
{
    /// This is the primary entrypoint for the Listener trait. listen
    /// is called exactly once, and is expected to spawn tasks for
    /// each incoming connection.
    fn listen<'a>(&'a mut self, app: Server<State>) -> BoxFuture<'a, io::Result<()>>;
}

pub(crate) fn is_transient_error(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::ConnectionRefused
        || e.kind() == io::ErrorKind::ConnectionAborted
        || e.kind() == io::ErrorKind::ConnectionReset
}
