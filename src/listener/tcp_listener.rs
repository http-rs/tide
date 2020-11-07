use super::is_transient_error;

use crate::listener::Listener;
use crate::{CancelationToken, log, Server};

use std::fmt::{self, Display, Formatter};

use async_std::net::{self, SocketAddr, TcpStream};
use async_std::prelude::*;
use async_std::{io, task};
use futures::future::{self, Either};

/// This represents a tide [Listener](crate::listener::Listener) that
/// wraps an [async_std::net::TcpListener]. It is implemented as an
/// enum in order to allow creation of a tide::listener::TcpListener
/// from a SocketAddr spec that has not yet been bound OR from a bound
/// TcpListener.
///
/// This is currently crate-visible only, and tide users are expected
/// to create these through [ToListener](crate::ToListener) conversions.
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
                format!("unable to connect to {:?}", addrs),
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

fn handle_tcp<State: Clone + Send + Sync + 'static>(app: Server<State>, stream: TcpStream) {
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
impl<State: Clone + Send + Sync + 'static> Listener<State> for TcpListener {
    async fn listen(&mut self, app: Server<State>, cancelation_token: CancelationToken) -> io::Result<()> {
        self.connect().await?;
        let listener = self.listener()?;
        crate::log::info!("Server listening on {}", self);

        let mut incoming = listener.incoming();

        'serve_loop:
        while let Either::Left(result) = future::select(incoming.next(), cancelation_token.clone()).await {
            match result.0 {
                Some(stream) => {
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
                },
                None => {
                    break 'serve_loop;
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
