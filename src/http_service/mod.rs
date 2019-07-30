//! HTTP Client Interface

use futures::future::BoxFuture;
use futures::io::AsyncRead;

use std::error::Error;
use std::fmt::{self, Debug};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

/// An HTTP Request type with a streaming body.
pub type Request = http::Request<Body>;

/// An HTTP Response type with a streaming body.
pub type Response = http::Response<Body>;

/// An abstract HTTP service.
pub trait HttpService: Debug + Unpin + Send + Sync + Clone + 'static {
    /// The associated error type.
    type Error: Error + Send + Sync;

    /// Perform a request.
    fn send(&self, req: Request) -> BoxFuture<'static, Result<Response, Self::Error>>;
}

/// The raw body of an http request or response.
///
/// A body is a stream of `Bytes` values, which are shared handles to byte buffers.
/// Both `Body` and `Bytes` values can be easily created from standard owned byte buffer types
/// like `Vec<u8>` or `String`, using the `From` trait.
pub struct Body {
    reader: Box<dyn AsyncRead + Unpin + Send + 'static>,
}

impl Body {
    /// Create a new empty body.
    pub fn empty() -> Self {
        Self {
            reader: Box::new(std::io::empty()),
        }
    }

    /// Create a new instance from a reader.
    pub fn from_reader(reader: Box<dyn AsyncRead + Unpin + Send + 'static>) -> Self {
        Self { reader }
    }
}

impl AsyncRead for Body {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Body").field("reader", &"<hidden>").finish()
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(vec: Vec<u8>) -> Body {
        Self {
            reader: Box::new(io::Cursor::new(vec)),
        }
    }
}

impl<R: AsyncRead + Unpin + Send + 'static> From<Box<R>> for Body {
    /// Converts an `AsyncRead` into a Body.
    fn from(reader: Box<R>) -> Self {
        Self { reader }
    }
}
