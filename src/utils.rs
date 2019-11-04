use std::pin::Pin;
use std::future::Future;

/// An owned dynamically typed [`Future`] for use in cases where you can't
/// statically type your result or need to add some indirection.
pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
