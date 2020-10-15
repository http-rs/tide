#[cfg(unix)]
use super::UnixListener;
use super::{Listener, TcpListener};
use crate::Server;

use async_std::io;
use std::fmt::{self, Display, Formatter};


/// This is an enum that contains variants for each of the listeners
/// that can be parsed from a string. This is used as the associated
/// Listener type for the string-parsing
/// [ToListener](crate::listener::ToListener) implementations
///
/// This is currently crate-visible only, and tide users are expected
/// to create these through [ToListener](crate::ToListener) conversions.
#[derive(Debug)]
pub enum ParsedListener {
    #[cfg(unix)]
    Unix(UnixListener),
    Tcp(TcpListener),
}

impl Display for ParsedListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => write!(f, "{}", u),
            Self::Tcp(t) => write!(f, "{}", t),
        }
    }
}

#[async_trait::async_trait]
impl<State, F> Listener<State, F> for ParsedListener
where
    State: Clone + Send + Sync + 'static,
    F: crate::listener::OnListen<State>,
{
    async fn listen_with(&mut self, app: Server<State>, f: F) -> io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.listen_with(app, f).await,
            Self::Tcp(t) => t.listen_with(app, f).await,
        }
    }
}
