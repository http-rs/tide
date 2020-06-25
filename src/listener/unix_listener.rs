use super::is_transient_error;

use crate::listener::Listener;
use crate::utils::BoxFuture;
use crate::{log, Server};

use std::fmt::{self, Display, Formatter};

use async_std::os::unix::net::{self, SocketAddr, UnixStream};
use async_std::prelude::*;
use async_std::{io, path::PathBuf, task};

#[derive(Debug)]
pub enum UnixListener {
    FromPath(PathBuf, Option<net::UnixListener>),
    FromListener(net::UnixListener),
}

impl UnixListener {
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self::FromPath(path.into(), None)
    }

    pub fn from_listener(listener: impl Into<net::UnixListener>) -> Self {
        Self::FromListener(listener.into())
    }

    fn listener(&self) -> io::Result<&net::UnixListener> {
        match self {
            Self::FromPath(_, Some(listener)) => Ok(listener),
            Self::FromListener(listener) => Ok(listener),
            Self::FromPath(path, None) => Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                format!("unable to connect {}", path.to_str().unwrap_or("[unknown]")),
            )),
        }
    }

    async fn connect(&mut self) -> io::Result<()> {
        if let Self::FromPath(path, listener @ None) = self {
            *listener = Some(net::UnixListener::bind(path).await?);
        }
        Ok(())
    }
}

fn unix_socket_addr_to_string(result: io::Result<SocketAddr>) -> Option<String> {
    result.ok().and_then(|addr| {
        if let Some(pathname) = addr.as_pathname().and_then(|p| p.canonicalize().ok()) {
            Some(format!("unix://{}", pathname.display()))
        } else {
            None
        }
    })
}

fn handle_unix<State: Send + Sync + 'static>(app: Server<State>, stream: UnixStream) {
    task::spawn(async move {
        let local_addr = unix_socket_addr_to_string(stream.local_addr());
        let peer_addr = unix_socket_addr_to_string(stream.peer_addr());

        let fut = async_h1::accept(stream, |mut req| async {
            req.set_local_addr(local_addr.as_ref());
            req.set_peer_addr(peer_addr.as_ref());
            app.respond(req).await
        });

        if let Err(error) = fut.await {
            log::error!("async-h1 error", { error: error.to_string() });
        }
    });
}

impl<State: Send + Sync + 'static> Listener<State> for UnixListener {
    fn listen<'a>(&'a mut self, app: Server<State>) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move {
            self.connect().await?;
            crate::log::info!("listening on {}", self);
            let listener = self.listener()?;
            let mut incoming = listener.incoming();

            while let Some(stream) = incoming.next().await {
                match stream {
                    Err(ref e) if is_transient_error(e) => continue,
                    Err(error) => {
                        let delay = std::time::Duration::from_millis(500);
                        crate::log::error!("Error: {}. Pausing for {:?}.", error, delay);
                        task::sleep(delay).await;
                        continue;
                    }

                    Ok(stream) => {
                        handle_unix(app.clone(), stream);
                    }
                };
            }

            Ok(())
        })
    }
}

impl Display for UnixListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FromListener(l) | Self::FromPath(_, Some(l)) => write!(
                f,
                "{}",
                unix_socket_addr_to_string(l.local_addr())
                    .as_deref()
                    .unwrap_or("unix://[unknown]")
            ),
            Self::FromPath(path, None) => {
                write!(f, "unix://{}", path.to_str().unwrap_or("[unknown]"))
            }
        }
    }
}
