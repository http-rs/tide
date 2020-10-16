use async_trait::async_trait;
use std::future::Future;
use std::io;

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
/// use tide::listener::ListenInfo;
///
/// let mut app = tide::new();
/// app.at("/").get(|_| async { Ok("Hello, world!") });
/// app.listen_with("127.0.0.1:8080", |info: ListenInfo| async move {
///     println!("started listening on {}!", info.connection());
///     Ok(())
/// }).await?;
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
