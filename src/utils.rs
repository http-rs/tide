use std::future::Future;
use std::pin::Pin;

/// An owned dynamically typed [`Future`] for use in cases where you can't
/// statically type your result or need to add some indirection.
pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
