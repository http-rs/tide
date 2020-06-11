use crate::listener::{Listener, ToListener};
use crate::utils::BoxFuture;
use crate::Server;

use std::fmt::{self, Debug, Display, Formatter};

use async_std::io;
use async_std::prelude::*;

/// MultiListener allows tide to listen on any number of transports
/// simultaneously (such as tcp ports, unix sockets, or tls).
///
/// # Example:
/// ```rust
/// fn main() -> Result<(), std::io::Error> {
///    async_std::task::block_on(async {
///        tide::log::start();
///        let mut app = tide::new();
///        app.at("/").get(|_| async { Ok("Hello, world!") });
///
///        let mut multi = tide::listener::MultiListener::new();
///        multi.add("127.0.0.1:8000")?;
///        multi.add(async_std::net::TcpListener::bind("127.0.0.1:8001").await?)?;
/// # if cfg!(unix) {
///        multi.add("unix://unix.socket")?;
/// # }
///    
/// # if false {
///        app.listen(multi).await?;
/// # }
///        Ok(())
///    })
///}
///```

pub struct MultiListener<State>(Vec<Box<dyn Listener<State>>>);

impl<State: Send + Sync + 'static> MultiListener<State> {
    /// creates a new MultiListener
    pub fn new() -> Self {
        Self(vec![])
    }

    /// Adds any [`ToListener`](crate::listener::ToListener) to this
    /// MultiListener. An error result represents a failure to convert
    /// the [`ToListener`](crate::listener::ToListener) into a
    /// [`Listener`](crate::listener::Listener).
    ///
    /// ```rust
    /// # fn main() -> std::io::Result<()> {
    /// let mut multi = tide::listener::MultiListener::new();
    /// multi.add("127.0.0.1:8000")?;
    /// multi.add(("localhost", 8001))?;
    /// multi.add(std::net::TcpListener::bind(("localhost", 8002))?)?;
    /// # std::mem::drop(tide::new().listen(multi)); // for the State generic
    /// # Ok(()) }
    /// ```
    pub fn add<TL: ToListener<State>>(&mut self, listener: TL) -> io::Result<()> {
        self.0.push(Box::new(listener.to_listener()?));
        Ok(())
    }

    /// `MultiListener::with` allows for chained construction of a MultiListener:
    /// ```rust,no_run
    /// # use tide::listener::MultiListener;
    /// # fn main() -> std::io::Result<()> { async_std::task::block_on(async move {
    /// # let app = tide::new();
    /// app.listen(
    ///     MultiListener::new()
    ///         .with("127.0.0.1:8080")
    ///         .with(async_std::net::TcpListener::bind("127.0.0.1:8081").await?),
    /// ).await?;
    /// #  Ok(()) }) }
    pub fn with<TL: ToListener<State>>(mut self, listener: TL) -> Self {
        self.add(listener).expect("Unable to add listener");
        self
    }

    /// from_iter allows for the construction of a new MultiListener
    /// from collections of [`ToListener`](ToListener)s.
    /// ```rust
    /// # use tide::listener::MultiListener;
    /// # fn main() -> std::io::Result<()> {
    /// let mut multi = MultiListener::from_iter(vec!["127.0.0.1:8000", "tcp://localhost:8001"])?;
    /// if cfg!(unix) {
    ///     multi.add("unix:///var/run/tide/socket")?;
    /// }
    /// # std::mem::drop(tide::new().listen(multi)); // for the State generic
    /// # Ok(()) }
    /// ```
    pub fn from_iter<TL: ToListener<State>>(vec: impl IntoIterator<Item = TL>) -> io::Result<Self> {
        let mut multi = Self::new();
        for listener in vec {
            multi.add(listener)?;
        }
        Ok(multi)
    }
}

impl<State: Send + Sync + 'static> Listener<State> for MultiListener<State> {
    fn connect<'a>(&'a mut self) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move {
            for listener in self.0.iter_mut() {
                listener.connect().await?;
            }
            Ok(())
        })
    }

    fn listen<'a>(&'a self, app: Server<State>) -> BoxFuture<'a, io::Result<()>> {
        let mut fut: Option<BoxFuture<'a, io::Result<()>>> = None;

        for listener in self.0.iter() {
            let app = app.clone();
            let listened = listener.listen(app);
            if let Some(f) = fut {
                fut = Some(Box::pin(f.race(listened)));
            } else {
                fut = Some(Box::pin(listened));
            }
        }

        fut.expect("at least one listener must be provided to a MultiListener")
    }
}

impl<State> Debug for MultiListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<State> Display for MultiListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = self
            .0
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(f, "{}", string)
    }
}
