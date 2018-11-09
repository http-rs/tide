//! Types for working directly with the bodies of requests and responses.
//!
//! This module includes types like `Json`, which can be used to automatically (de)serialize bodies
//! using `serde_json`.

use futures::{compat::Compat01As03, prelude::*, future::FutureObj, stream::StreamObj};
use pin_utils::pin_mut;
use http::status::StatusCode;

use crate::{Extract, IntoResponse, RouteMatch, Request, Response};

/// The raw contents of an http request or response.
///
/// A body is a stream of `BodyChunk`s, which are essentially `Vec<u8>` values.
/// Both `Body` and `BodyChunk` values can be easily created from standard byte buffer types,
/// using the `From` trait.
pub struct Body {
    inner: BodyInner,
}

type BodyStream = StreamObj<'static, Result<BodyChunk, Error>>;
type Error = Box<dyn std::error::Error + Send + Sync>;
pub struct BodyChunk(hyper::Chunk);

impl BodyChunk {
    pub fn as_bytes(&self) -> &[u8] {
        (*self.0).as_ref()
    }
}

impl From<Vec<u8>> for BodyChunk {
    fn from(v: Vec<u8>) -> Self {
        BodyChunk(v.into())
    }
}

impl From<String> for BodyChunk {
    fn from(v: String) -> Self {
        BodyChunk(v.into())
    }
}

enum BodyInner {
    Streaming(BodyStream),
    Fixed(Vec<u8>),
}

impl Body {
    /// Create an empty body.
    pub fn empty() -> Self {
        Body {
            inner: BodyInner::Fixed(Vec::new()),
        }
    }

    /// Collect the full contents of the body into a vector.
    ///
    /// This method is asynchronous because, in general, it requires reading an async
    /// stream of `BodyChunk` values.
    pub async fn to_vec(&mut self) -> Result<Vec<u8>, Error> {
        match &mut self.inner {
            BodyInner::Streaming(s) => {
                let mut bytes = Vec::new();
                pin_mut!(s);
                while let Some(chunk) = await!(s.next()) {
                    // TODO: do something more robust than `unwrap`
                    bytes.extend(chunk?.as_bytes());
                }
                Ok(bytes)
            }
            BodyInner::Fixed(v) => Ok(v.clone()),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self {
        Self {
            inner: BodyInner::Fixed(v),
        }
    }
}

impl From<hyper::Body> for Body {
    fn from(body: hyper::Body) -> Body {
        // TODO: handle chunk-level errors
        let stream = Compat01As03::new(body).map(|c| match c {
            Ok(chunk) => Ok(BodyChunk(chunk)),
            Err(e) => {
                let e: Error = Box::new(e);
                Err(e)
            }
        });
        Body {
            inner: BodyInner::Streaming(StreamObj::new(Box::new(stream))),
        }
    }
}

impl From<BodyChunk> for hyper::Chunk {
    fn from(chunk: BodyChunk) -> hyper::Chunk {
        chunk.0
    }
}

impl Into<hyper::Body> for Body {
    fn into(self) -> hyper::Body {
        match self.inner {
            BodyInner::Fixed(v) => v.into(),
            BodyInner::Streaming(s) => hyper::Body::wrap_stream(s.compat()),
        }
    }
}

/// A wrapper for json (de)serialization of bodies.
///
/// This type is usable both as an extractor (argument to an endpoint) and as a response
/// (return value from an endpoint).
pub struct Json<T>(pub T);

impl<T: Send + serde::de::DeserializeOwned + 'static, S: 'static> Extract<S> for Json<T> {
    // Note: cannot use `existential type` here due to ICE
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &RouteMatch<'_>,
    ) -> Self::Fut {
        let mut body = std::mem::replace(req.body_mut(), Body::empty());
        FutureObj::new(Box::new(async move {
            fn mk_err<T>(_: T) -> Response { StatusCode::BAD_REQUEST.into_response() }
            let body = await!(body.to_vec()).map_err(mk_err)?;
            let json: T = serde_json::from_slice(&body).map_err(mk_err)?;
            Ok(Json(json))
        }))
    }
}

impl<T: 'static + Send + serde::Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        // TODO: think about how to handle errors
        http::Response::builder()
            .status(http::status::StatusCode::OK)
            .header("Content-Type", "Application/json")
            .body(Body::from(serde_json::to_vec(&self.0).unwrap()))
            .unwrap()
    }
}

impl<S: 'static> Extract<S> for String {
  type Fut = FutureObj<'static, Result<Self, Response>>;

  fn extract(
    data: &mut S,
    req: &mut Request,
    params: &RouteMatch<'_>,
  ) -> Self::Fut {
    let mut body = std::mem::replace(req.body_mut(), Body::empty());

    FutureObj::new(Box::new(async move {
      fn mk_err<T>(_: T) -> Response { StatusCode::BAD_REQUEST.into_response() }
      let body = await!(body.to_vec().map_err(mk_err))?;
      let string = String::from_utf8(body).map_err(mk_err)?;
      Ok(string)
    }))
  }
}

impl<S: 'static> Extract<S> for Vec<u8> {
  type Fut = FutureObj<'static, Result<Self, Response>>;

  fn extract(
    data: &mut S,
    req: &mut Request,
    params: &RouteMatch<'_>,
  ) -> Self::Fut {
    let mut body = std::mem::replace(req.body_mut(), Body::empty());

    FutureObj::new(Box::new(async move {
      fn mk_err<T>(_: T) -> Response { StatusCode::BAD_REQUEST.into_response() }
      let body = await!(body.to_vec().map_err(mk_err))?;
      Ok(body)
    }))
  }
}


