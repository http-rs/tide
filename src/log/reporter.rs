use crate::listener::{ConnectionInfo, Report};
use async_trait::async_trait;
use std::io;

/// Empty `Report` impl.
#[derive(Clone, Debug)]
pub struct Reporter;

#[async_trait]
impl Report for Reporter {
    async fn call(&self, info: ConnectionInfo) -> io::Result<()> {
        log::info!("Server listening on {}", info.connection());
        Ok(())
    }
}
