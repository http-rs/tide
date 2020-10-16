use crate::listener::{ConnectionInfo, OnListen};
use async_trait::async_trait;
use std::io;

/// Empty `OnListen` impl.
#[derive(Clone, Debug)]
pub struct OnConnect;

#[async_trait]
impl OnListen for OnConnect {
    async fn call(&self, info: ConnectionInfo) -> io::Result<()> {
        log::info!("Server listening on {}", info.connection());
        Ok(())
    }
}
