use crate::listener::{Listener, ToListener};
use crate::Server;

use std::fmt::{self, Debug, Display, Formatter};
use std::future::Future;

use async_std::io;
use async_std::sync::{Arc, Barrier};
use futures_util::stream::{futures_unordered::FuturesUnordered, StreamExt};

/// ConcurrentListener allows tide to listen on any number of transports
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
///        let mut listener = tide::listener::ConcurrentListener::new();
///        listener.add("127.0.0.1:8000")?;
///        listener.add(async_std::net::TcpListener::bind("127.0.0.1:8001").await?)?;
/// # if cfg!(unix) {
///        listener.add("http+unix://unix.socket")?;
/// # }
///    
/// # if false {
///        app.listen(listener).await?;
/// # }
///        Ok(())
///    })
///}
///```

#[derive(Default)]
pub struct ConcurrentListener<State, F, Fut>(Vec<Box<dyn Listener<State, F, Fut>>>);

impl<State, F, Fut> ConcurrentListener<State, F, Fut>
where
    State: Clone + Send + Sync + 'static,
    F: Fn(Server<State>) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = io::Result<Server<State>>> + Send + Sync + 'static,
{
    /// creates a new ConcurrentListener
    pub fn new() -> Self {
        Self(vec![])
    }

    /// Adds any [`ToListener`](crate::listener::ToListener) to this
    /// ConcurrentListener. An error result represents a failure to convert
    /// the [`ToListener`](crate::listener::ToListener) into a
    /// [`Listener`](crate::listener::Listener).
    ///
    /// ```rust
    /// # fn main() -> std::io::Result<()> {
    /// let mut listener = tide::listener::ConcurrentListener::new();
    /// listener.add("127.0.0.1:8000")?;
    /// listener.add(("localhost", 8001))?;
    /// listener.add(std::net::TcpListener::bind(("localhost", 8002))?)?;
    /// # std::mem::drop(tide::new().listen(listener)); // for the State generic
    /// # Ok(()) }
    /// ```
    pub fn add<L: ToListener<State, F, Fut>>(&mut self, listener: L) -> io::Result<()> {
        self.0.push(Box::new(listener.to_listener()?));
        Ok(())
    }

    /// `ConcurrentListener::with_listener` allows for chained construction of a ConcurrentListener:
    /// ```rust,no_run
    /// # use tide::listener::ConcurrentListener;
    /// # fn main() -> std::io::Result<()> { async_std::task::block_on(async move {
    /// # let app = tide::new();
    /// app.listen(
    ///     ConcurrentListener::new()
    ///         .with_listener("127.0.0.1:8080")
    ///         .with_listener(async_std::net::TcpListener::bind("127.0.0.1:8081").await?),
    /// ).await?;
    /// #  Ok(()) }) }
    pub fn with_listener<L>(mut self, listener: L) -> Self
    where
        L: ToListener<State, F, Fut>,
    {
        self.add(listener).expect("Unable to add listener");
        self
    }
}

#[async_trait::async_trait]
impl<State, F, Fut> Listener<State, F, Fut> for ConcurrentListener<State, F, Fut>
where
    State: Clone + Send + Sync + 'static,
    F: Fn(Server<State>) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = io::Result<Server<State>>> + Send + Sync + 'static,
{
    async fn listen_with(&mut self, app: Server<State>, f: F) -> io::Result<()> {
        let mut futures_unordered = FuturesUnordered::new();

        let barrier = Arc::new(Barrier::new(self.0.len()));

        for listener in self.0.iter_mut() {
            let app = app.clone();
            let c = barrier.clone();

            // Call `listen_with` on each individual listener, and after they
            // all started listen invoke `f` exactly once.
            futures_unordered.push(listener.listen_with(
                app,
                Box::new(|app| async move {
                    let res = c.wait().await;
                    if res.is_leader() {
                        app = f(app).await?;
                    }
                    Ok(app)
                }),
            ));
        }

        while let Some(result) = futures_unordered.next().await {
            result?;
        }
        Ok(())
    }
}

impl<State, F, Fut> Debug for ConcurrentListener<State, F, Fut> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<State, F, Fut> Display for ConcurrentListener<State, F, Fut> {
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
