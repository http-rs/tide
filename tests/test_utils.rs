use portpicker::pick_unused_port;

use std::future::Future;
use std::pin::Pin;

/// An owned dynamically typed [`Future`] for use in cases where you can't
/// statically type your result or need to add some indirection.
#[allow(dead_code)]
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Find an unused port.
#[allow(dead_code)]
pub async fn find_port() -> u16 {
    pick_unused_port().expect("No ports free")
}
