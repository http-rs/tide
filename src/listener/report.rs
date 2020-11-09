use async_std::sync::Sender;
use async_trait::async_trait;
use std::future::Future;
use std::io;
use std::pin::Pin;

use crate::listener::ListenInfo;

/// Report connection info for each listener through `Server::listen_with`.
///
/// When a listener successfully starts listening, it should create an instance
/// of `ListenInfo` and invoke `Report::call`. When using listeners such as
/// `ConcurrentListener` `Report::call` may be invoked multiple times.
///
/// # Examples
///
/// ```no_run
/// # use async_std::task::block_on;
/// # fn main() -> Result<(), std::io::Error> { block_on(async {
/// #
/// let mut app = tide::new();
/// app.at("/").get(|_| async { Ok("Hello, world!") });
/// let listener = app.bind("127.0.0.1:8080").await?;
/// println!("started listening on {}!", listener.info().connection());
/// listener.accept().await?;
/// #
/// # Ok(()) }) }
/// ```
#[async_trait]
pub trait Report: Clone + Send + Sync + 'static {
    async fn call(&self, server: ListenInfo) -> io::Result<()>;
}

#[async_trait]
impl<F, Fut> Report for F
where
    F: Clone + Send + Sync + 'static + Fn(ListenInfo) -> Fut,
    Fut: Future<Output = io::Result<()>> + Send + 'static,
{
    async fn call(&self, info: ListenInfo) -> io::Result<()> {
        let fut = (self)(info);
        Ok(fut.await?)
    }
}

/// A `log`-based `Report` implementation.
///
/// This is the default reporter used in `Server::listen`.
#[derive(Clone, Debug)]
pub struct Reporter {
    sender: Sender<ListenInfo>,
}

impl Reporter {
    /// Create a new instance.
    pub fn new(sender: Sender<ListenInfo>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl Report for Reporter {
    async fn call(&self, info: ListenInfo) -> io::Result<()> {
        log::info!("Server listening on {}", info.connection());
        Ok(())
    }
}

/// A bound server.
pub struct Bound {
    info: ListenInfo,
    fut: Pin<Box<dyn std::future::Future<Output = io::Result<()>> + Send + 'static>>,
}

impl std::fmt::Debug for Bound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bound").field(&"info", &self.info).finish()
    }
}

impl Bound {
    pub(crate) fn new(
        info: ListenInfo,
        fut: Pin<Box<dyn std::future::Future<Output = std::io::Result<()>> + Send + 'static>>,
    ) -> Self {
        Self { info, fut }
    }

    pub fn info(&self) -> &ListenInfo {
        &self.info
    }

    pub async fn accept(self) -> io::Result<()> {
        self.fut.await
    }
}
