use crate::listener::{ListenInfo, Report};
use async_trait::async_trait;
use std::io;

/// A `log`-based `Report` implementation.
///
/// This is the default reporter used in `Server::listen`.
#[derive(Clone, Debug)]
pub struct Reporter;

#[async_trait]
impl Report for Reporter {
    async fn call(&self, info: ListenInfo) -> io::Result<()> {
        log::info!("Server listening on {}", info.connection());
        Ok(())
    }
}
