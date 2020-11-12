use super::Listener;
use async_std::io;

/// ToListener represents any type that can be converted into a
/// [`Listener`](crate::listener::Listener).  Any type that implements
/// ToListener can be passed to [`Server::listen`](crate::Server::listen) or
/// added to a [`ConcurrentListener`](crate::listener::ConcurrentListener)
///
/// # Example strings on all platforms include:
/// * `tcp://localhost:8000`
/// * `tcp://0.0.0.0` (binds to port 80 by default)
/// * `http://localhost:8000` (http is an alias for tcp)
/// * `http://127.0.0.1:8000` (or `0.0.0.0`, or some specific bindable ip)
/// * `127.0.0.1:3000` (or any string that can be parsed as a [SocketAddr](std::net::SocketAddr))
/// * `[::1]:1213` (an ipv6 [SocketAddr](std::net::SocketAddr))
///
/// # Strings supported only on `cfg(unix)` platforms:
/// * `http+unix:///var/run/tide/socket` (absolute path)
/// * `http+unix://socket` (relative path)
/// * `http+unix://./socket.file` (also relative path)
/// * `http+unix://../socket` (relative path)
///
/// # String supported only on windows:
/// * `:3000` (binds to port 3000)
///
/// # Specifying multiple listeners:
/// To bind to any number of listeners concurrently:
/// ```rust,no_run
/// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
/// # let app = tide::new();
/// app.listen(vec!["tcp://localhost:8000", "tcp://localhost:8001"]).await?;
/// # Ok(()) }) }
/// ```
///
/// # Multiple socket resolution
/// If a TCP listener resolves to multiple socket addresses, tide will
/// bind to the first successful one. For example, on ipv4+ipv6
/// systems, `tcp://localhost:1234` resolves both to `127.0.0.1:1234`
/// (v4) as well as `[::1]:1234` (v6). The order that these are
/// attempted is platform-determined. To listen on all of the addresses, use
/// ```rust,no_run
/// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
/// # let app = tide::new();
/// use std::net::ToSocketAddrs;
/// app.listen("localhost:8000".to_socket_addrs()?.collect::<Vec<_>>()).await?;
/// # Ok(()) }) }
/// ```
/// # Other implementations
/// See below for additional provided implementations of ToListener.
pub trait ToListener<State: Clone + Send + Sync + 'static> {
    /// What listener are we converting into?
    type Listener: Listener<State>;

    /// Transform self into a
    /// [`Listener`](crate::listener::Listener). Unless self is
    /// already bound/connected to the underlying io, converting to a
    /// listener does not initiate a connection. An Err return
    /// indicates an unsuccessful conversion to a listener, not an
    /// unsuccessful bind attempt.
    fn to_listener(self) -> io::Result<Self::Listener>;
}
