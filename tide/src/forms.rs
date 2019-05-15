use http_service::Body;
use multipart::server::Multipart;
use std::io::Cursor;

use crate::{
    error::{BoxTryFuture, ResultExt},
    Context, Response,
};

/// An extension trait for `Context`, providing form extraction.
pub trait ExtractForms {
    /// Asynchronously extract the entire body as a single form.
    fn body_form<T: serde::de::DeserializeOwned>(&mut self) -> BoxTryFuture<T>;

    /// Asynchronously extract the entire body as a multipart form.
    fn body_multipart(&mut self) -> BoxTryFuture<Multipart<Cursor<Vec<u8>>>>;
}

impl<State: Send + Sync + 'static> ExtractForms for Context<State> {
    fn body_form<T: serde::de::DeserializeOwned>(&mut self) -> BoxTryFuture<T> {
        let body = self.take_body();
        box_async! {
            let body = body.into_vec().await.client_err()?;
            Ok(serde_urlencoded::from_bytes(&body).map_err(|e| err_fmt!("could not decode form: {}", e)).client_err()?)
        }
    }

    fn body_multipart(&mut self) -> BoxTryFuture<Multipart<Cursor<Vec<u8>>>> {
        const BOUNDARY: &str = "boundary=";
        let boundary = self.headers().get("content-type").and_then(|ct| {
            let ct = ct.to_str().ok()?;
            let idx = ct.find(BOUNDARY)?;
            Some(ct[idx + BOUNDARY.len()..].to_string())
        });

        let body = self.take_body();

        box_async! {
            let body = body.into_vec().await.client_err()?;
            let boundary = boundary.ok_or_else(|| err_fmt!("no boundary found")).client_err()?;
            Ok(Multipart::with_body(Cursor::new(body), boundary))
        }
    }
}

/// Encode `t` as a form response.
pub fn form<T: serde::Serialize>(t: T) -> Response {
    // TODO: think about how to handle errors
    http::Response::builder()
        .status(http::status::StatusCode::OK)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(
            serde_urlencoded::to_string(&t).unwrap().into_bytes(),
        ))
        .unwrap()
}
