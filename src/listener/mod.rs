//! Types that represent HTTP transports and binding

mod concurrent_listener;
mod failover_listener;
#[cfg(feature = "h1-server")]
mod parsed_listener;
#[cfg(feature = "h1-server")]
mod tcp_listener;
mod to_listener;
#[cfg(feature = "h1-server")]
mod to_listener_impls;
#[cfg(all(unix, feature = "h1-server"))]
mod unix_listener;

use std::fmt::{Debug, Display};

use crate::Server;
use async_std::io;

pub use concurrent_listener::ConcurrentListener;
pub use failover_listener::FailoverListener;
pub use to_listener::ToListener;

#[cfg(feature = "h1-server")]
pub(crate) use parsed_listener::ParsedListener;
#[cfg(feature = "h1-server")]
pub(crate) use tcp_listener::TcpListener;
#[cfg(all(unix, feature = "h1-server"))]
pub(crate) use unix_listener::UnixListener;

/// The Listener trait represents an implementation of http transport
/// for a tide application. In order to provide a Listener to tide,
/// you will also need to implement at least one [`ToListener`](crate::listener::ToListener) that
/// outputs your Listener type.
#[async_trait::async_trait]
pub trait Listener<State>: Debug + Display + Send + Sync + 'static
where
    State: Send + Sync + 'static,
{
    /// Bind the listener to the specified address.
    async fn bind(&mut self, app: Server<State>) -> io::Result<()>;

    /// This is the primary entrypoint for the Listener trait. listen
    /// is called exactly once, and is expected to spawn tasks for
    /// each incoming connection.
    ///
    /// Accept a new incoming connection from this listener.
    async fn accept(&mut self) -> io::Result<()>;
}

#[async_trait::async_trait]
impl<L, State> Listener<State> for Box<L>
where
    L: Listener<State>,
    State: Send + Sync + 'static,
{
    async fn bind(&mut self, app: Server<State>) -> io::Result<()> {
        self.as_mut().bind(app).await
    }

    async fn accept(&mut self) -> io::Result<()> {
        self.as_mut().accept().await
    }
}

/// crate-internal shared logic used by tcp and unix listeners to
/// determine if an io::Error needs a backoff delay. Transient error
/// types do not require a delay.
#[cfg(feature = "h1-server")]
pub(crate) fn is_transient_error(e: &io::Error) -> bool {
    use io::ErrorKind::*;

    matches!(
        e.kind(),
        ConnectionRefused | ConnectionAborted | ConnectionReset
    )
}
