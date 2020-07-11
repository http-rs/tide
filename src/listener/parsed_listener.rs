#[cfg(unix)]
use super::UnixListener;
use super::{Listener, TcpListener};
use crate::Server;
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

#[async_trait::async_trait]
impl<State: Send + Sync + 'static> Listener<State> for ParsedListener {
    async fn listen(&mut self, app: Server<State>) -> io::Result<()> {
        match self {
            #[cfg(unix)]
            Self::Unix(u) => u.listen(app).await,
            Self::Tcp(t) => t.listen(app).await,
        }
    }
}
