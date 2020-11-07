use crate::listener::{Listener, ToListener};
use crate::{CancelationToken, Server};

use std::fmt::{self, Debug, Display, Formatter};

use async_std::{io, task};

/// FailoverListener allows tide to attempt to listen in a sequential
/// order to any number of ports/addresses. The first successful
/// listener is used.
///
/// # Example:
/// ```rust
/// fn main() -> Result<(), std::io::Error> {
///    async_std::task::block_on(async {
///        tide::log::start();
///        let mut app = tide::new();
///        app.at("/").get(|_| async { Ok("Hello, world!") });
///
///        let mut listener = tide::listener::FailoverListener::new();
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
pub struct FailoverListener<State>(Vec<Box<dyn Listener<State>>>);

impl<State: Clone + Send + Sync + 'static> FailoverListener<State> {
    /// creates a new FailoverListener
    pub fn new() -> Self {
        Self(vec![])
    }

    /// Adds any [`ToListener`](crate::listener::ToListener) to this
    /// FailoverListener. An error result represents a failure to convert
    /// the [`ToListener`](crate::listener::ToListener) into a
    /// [`Listener`](crate::listener::Listener).
    ///
    /// ```rust
    /// # fn main() -> std::io::Result<()> {
    /// let mut listener = tide::listener::FailoverListener::new();
    /// listener.add("127.0.0.1:8000")?;
    /// listener.add(("localhost", 8001))?;
    /// # if cfg!(unix) {
    /// listener.add("http+unix:///var/run/tide")?;
    /// # }
    /// # std::mem::drop(tide::new().listen(listener)); // for the State generic
    /// # Ok(()) }
    /// ```
    pub fn add<TL: ToListener<State>>(&mut self, listener: TL) -> io::Result<()> {
        self.0.push(Box::new(listener.to_listener()?));
        Ok(())
    }

    /// `FailoverListener::with_listener` allows for chained construction of a FailoverListener:
    /// ```rust,no_run
    /// # use tide::listener::FailoverListener;
    /// # fn main() -> std::io::Result<()> { async_std::task::block_on(async move {
    /// # let app = tide::new();
    /// app.listen(
    ///     FailoverListener::new()
    ///         .with_listener("127.0.0.1:8080")
    ///         .with_listener(("localhost", 8081)),
    /// ).await?;
    /// #  Ok(()) }) }
    pub fn with_listener<TL: ToListener<State>>(mut self, listener: TL) -> Self {
        self.add(listener).expect("Unable to add listener");
        self
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> Listener<State> for FailoverListener<State> {
    async fn listen(&mut self, app: Server<State>, cancelation_token: CancelationToken) -> io::Result<()> {

        let mut cancelation_tokens = Vec::new();

        for listener in self.0.iter_mut() {
            let app = app.clone();
            let sub_cancelation_token = CancelationToken::new();
            match listener.listen(app, sub_cancelation_token.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    crate::log::info!("unable to listen", {
                        listener: listener.to_string(),
                        error: e.to_string()
                    });
                }
            }
            cancelation_tokens.push(sub_cancelation_token);
        }

        task::spawn(async move {
            cancelation_token.await;
            for sub_cancelation_token in cancelation_tokens.iter_mut() {
                sub_cancelation_token.complete();
            }
        });

        Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "unable to bind to any supplied listener spec",
        ))
    }
}

impl<State> Debug for FailoverListener<State> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<State> Display for FailoverListener<State> {
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
