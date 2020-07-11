use super::is_transient_error;

use crate::listener::Listener;
use crate::{log, Server};

use std::fmt::{self, Display, Formatter};

use async_std::net::{self, SocketAddr, TcpStream};
use async_std::prelude::*;
use async_std::{io, task};

#[derive(Debug)]
pub enum TcpListener {
    FromListener(net::TcpListener),
    FromAddrs(Vec<SocketAddr>, Option<net::TcpListener>),
}

impl TcpListener {
    pub fn from_addrs(addrs: Vec<SocketAddr>) -> Self {
        Self::FromAddrs(addrs, None)
    }

    pub fn from_listener(tcp_listener: impl Into<net::TcpListener>) -> Self {
        Self::FromListener(tcp_listener.into())
    }

    fn listener(&self) -> io::Result<&net::TcpListener> {
        match self {
            Self::FromAddrs(_, Some(listener)) => Ok(listener),
            Self::FromListener(listener) => Ok(listener),
            Self::FromAddrs(addrs, None) => Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                format!("unable to connect {:?}", addrs),
            )),
        }
    }

    async fn connect(&mut self) -> io::Result<()> {
        if let Self::FromAddrs(addrs, listener @ None) = self {
            *listener = Some(net::TcpListener::bind(addrs.as_slice()).await?);
        }
        Ok(())
    }
}

fn handle_tcp<State: Send + Sync + 'static>(app: Server<State>, stream: TcpStream) {
    task::spawn(async move {
        let local_addr = stream.local_addr().ok();
        let peer_addr = stream.peer_addr().ok();

        let fut = async_h1::accept(stream, |mut req| async {
            req.set_local_addr(local_addr);
            req.set_peer_addr(peer_addr);
            app.respond(req).await
        });

        if let Err(error) = fut.await {
            log::error!("async-h1 error", { error: error.to_string() });
        }
    });
}

#[async_trait::async_trait]
impl<State: Send + Sync + 'static> Listener<State> for TcpListener {
    async fn listen(&mut self, app: Server<State>) -> io::Result<()> {
        self.connect().await?;
        let listener = self.listener()?;
        crate::log::info!("Server listening on {}", self);

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
                    handle_tcp(app.clone(), stream);
                }
            };
        }
        Ok(())
    }
}

impl Display for TcpListener {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::FromListener(l) | Self::FromAddrs(_, Some(l)) => write!(
                f,
                "http://{}",
                l.local_addr()
                    .ok()
                    .map(|a| a.to_string())
                    .as_deref()
                    .unwrap_or("[unknown]")
            ),
            Self::FromAddrs(addrs, None) => write!(
                f,
                "{}",
                addrs
                    .iter()
                    .map(|a| format!("http://{}", a))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
