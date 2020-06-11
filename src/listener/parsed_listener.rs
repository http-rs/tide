#[cfg(unix)]
use super::UnixListener;
use super::{Listener, TcpListener};
use crate::{utils::BoxFuture, Server};
use async_std::io;
use std::fmt::{self, Display, Formatter};

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

impl<State: Send + Sync + 'static> Listener<State> for ParsedListener {
    fn connect<'a>(&'a mut self) -> BoxFuture<'a, io::Result<()>> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => Listener::<State>::connect(u),
            Self::Tcp(t) => Listener::<State>::connect(t),
        }
    }

    fn listen<'a>(&'a self, app: Server<State>) -> BoxFuture<'a, io::Result<()>> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.listen(app),
            Self::Tcp(t) => t.listen(app),
        }
    }
}
