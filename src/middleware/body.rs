use async_std::io::prelude::*;
use async_std::prelude::*;
use async_std::task::{ready, Context, Poll};

use std::pin::Pin;

/// An HTTP Request body.
///
/// This type wraps a `Stream<Output = Bytes>` and converts it into `AsyncRead`.
pub struct RequestBody {
    byte_stream: http_service::Body,
}

/// An HTTP Response body.
///
/// This type wraps an `AsyncRead` and converts it into `Stream<Output = Bytes>`.
#[derive(Debug)]
pub struct ResponseBody {
    reader: Box<dyn Read + Unpin + Send + 'static>,
}

impl ResponseBody {
    /// Create a new empty body.
    pub fn empty() -> Self {
        Self {
            body: http_service::Body::empty(),
        }
    }
}

impl<R: Read + Unpin> Stream for ResponseBody {
    type Item = Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync + 'static>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // This is not at all efficient, but that's okay for now.
        let mut buf = vec![];
        let read = ready!(Pin::new(&mut self.reader).poll_read(cx, &mut buf))?;
        if read == 0 {
            return Poll::Ready(None);
        } else {
            buf.shrink_to_fit();
            let chunk = bytes::Bytes::from(buf);
            Poll::Ready(Some(Ok(chunk)))
        }
    }
}

/// A type that wraps an `Read` into a `Stream` of `hyper::Chunk`. Used for writing data to a
/// Hyper response.
struct ChunkStream<R: Read> {
    reader: R,
}
