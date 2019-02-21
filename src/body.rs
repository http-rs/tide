//! Types for working directly with the bodies of requests and responses.
//!
//! This module includes types like `Json`, which can be used to automatically (de)serialize bodies
//! using `serde_json`.
//!
//! # Examples
//!
//! Read/Write `Strings` and `Bytes` from/to bodies:
//!
//! ```rust, no_run
//! # #![feature(async_await, futures_api)]
//! use tide::body;
//!
//! async fn echo_string(msg: body::Str) -> String {
//!     println!("String: {}", *msg);
//!     format!("{}", *msg)
//! }
//!
//! async fn echo_string_lossy(msg: body::StrLossy) -> String {
//!     println!("String: {}", *msg);
//!     format!("{}", *msg)
//! }
//!
//! async fn echo_bytes(msg: body::Bytes) -> body::Bytes {
//!     println!("Bytes: {:?}", *msg);
//!     msg
//! }
//!
//! # fn main() {
//! #    let mut app = tide::App::new(());
//! #
//! app.at("/echo/string").post(echo_string);
//! app.at("/echo/string_lossy").post(echo_string_lossy);
//! app.at("/echo/bytes").post(echo_bytes);
//!
//! #    app.serve();
//! # }
//!
//! ```
//!
//! Using `serde_json` to automatically (de)serialize bodies into/from structs:
//!
//! ```rust, no_run
//! # #![feature(async_await, futures_api)]
//! #[macro_use]
//! extern crate serde_derive;
//! use tide::body;
//!
//! #[derive(Serialize, Deserialize, Clone, Debug)]
//! struct Message {
//!     author: Option<String>,
//!     contents: String,
//! }
//!
//! async fn echo_json(msg: body::Json<Message>) -> body::Json<Message> {
//!     println!("JSON: {:?}", *msg);
//!     msg
//! }
//!
//! async fn echo_form(msg: body::Form<Message>) -> body::Form<Message> {
//!     println!("Form: {:?}", *msg);
//!     msg
//! }
//!
//! # fn main() {
//! #    let mut app = tide::App::new(());
//! #
//! app.at("/echo/json").post(echo_json);
//! app.at("/echo/form").post(echo_form);
//! #
//! #    app.serve();
//! # }
//!
//! ```
//!
use futures::future::FutureObj;
use http::status::StatusCode;
use http_service::Body;
use multipart::server::Multipart;
use std::io::Cursor;
use std::ops::{Deref, DerefMut};

use crate::{configuration::Store, Extract, IntoResponse, Request, Response, RouteMatch};

// Small utility function to return a stamped error when we cannot parse a request body
fn mk_err<T>(_: T) -> Response {
    StatusCode::BAD_REQUEST.into_response()
}

/// A wrapper for multipart form
///
/// This type is useable as an extractor (argument to an endpoint) for getting
/// a Multipart type defined in the multipart crate
pub struct MultipartForm(pub Multipart<Cursor<Vec<u8>>>);

impl<S: 'static> Extract<S> for MultipartForm {
    // Note: cannot use `existential type` here due to ICE
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        // https://stackoverflow.com/questions/43424982/how-to-parse-multipart-forms-using-abonander-multipart-with-rocket

        const BOUNDARY: &str = "boundary=";
        let boundary = req.headers().get("content-type").and_then(|ct| {
            let ct = ct.to_str().ok()?;
            let idx = ct.find(BOUNDARY)?;
            Some(ct[idx + BOUNDARY.len()..].to_string())
        });

        let body = std::mem::replace(req.body_mut(), Body::empty());

        FutureObj::new(Box::new(
            async move {
                let body = await!(body.into_vec()).map_err(mk_err)?;
                let boundary = boundary.ok_or(()).map_err(mk_err)?;
                let mp = Multipart::with_body(Cursor::new(body), boundary);
                Ok(MultipartForm(mp))
            },
        ))
    }
}

impl Deref for MultipartForm {
    type Target = Multipart<Cursor<Vec<u8>>>;
    fn deref(&self) -> &Multipart<Cursor<Vec<u8>>> {
        &self.0
    }
}

impl DerefMut for MultipartForm {
    fn deref_mut(&mut self) -> &mut Multipart<Cursor<Vec<u8>>> {
        &mut self.0
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
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let body = std::mem::replace(req.body_mut(), Body::empty());
        FutureObj::new(Box::new(
            async move {
                let body = await!(body.into_vec()).map_err(mk_err)?;
                let json: T = serde_json::from_slice(&body).map_err(mk_err)?;
                Ok(Json(json))
            },
        ))
    }
}

impl<T: Send + serde::Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        // TODO: think about how to handle errors
        http::Response::builder()
            .status(http::status::StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&self.0).unwrap()))
            .unwrap()
    }
}

impl<T> Deref for Json<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// A wrapper for form encoded (application/x-www-form-urlencoded) (de)serialization of bodies.
///
/// This type is usable both as an extractor (argument to an endpoint) and as a response
/// (return value from an endpoint), though returning a response with form data is uncommon
/// and probably not good practice.
pub struct Form<T>(pub T);

impl<T: Send + serde::de::DeserializeOwned + 'static, S: 'static> Extract<S> for Form<T> {
    // Note: cannot use `existential type` here due to ICE
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let body = std::mem::replace(req.body_mut(), Body::empty());
        FutureObj::new(Box::new(
            async move {
                let body = await!(body.into_vec()).map_err(mk_err)?;
                let data: T = serde_qs::from_bytes(&body).map_err(mk_err)?;
                Ok(Form(data))
            },
        ))
    }
}

impl<T: 'static + Send + serde::Serialize> IntoResponse for Form<T> {
    fn into_response(self) -> Response {
        // TODO: think about how to handle errors
        http::Response::builder()
            .status(http::status::StatusCode::OK)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(
                serde_qs::to_string(&self.0).unwrap().into_bytes(),
            ))
            .unwrap()
    }
}

impl<T> Deref for Form<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Form<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct Str(pub String);

impl<S: 'static> Extract<S> for Str {
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let body = std::mem::replace(req.body_mut(), Body::empty());

        FutureObj::new(Box::new(
            async move {
                let body = await!(body.into_vec()).map_err(mk_err)?;
                let string = String::from_utf8(body).map_err(mk_err)?;
                Ok(Str(string))
            },
        ))
    }
}

impl Deref for Str {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl DerefMut for Str {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

pub struct StrLossy(pub String);

impl<S: 'static> Extract<S> for StrLossy {
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let body = std::mem::replace(req.body_mut(), Body::empty());

        FutureObj::new(Box::new(
            async move {
                let body = await!(body.into_vec()).map_err(mk_err)?;
                let string = String::from_utf8_lossy(&body).to_string();
                Ok(StrLossy(string))
            },
        ))
    }
}

impl Deref for StrLossy {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl DerefMut for StrLossy {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

pub struct Bytes(pub Vec<u8>);

impl<S: 'static> Extract<S> for Bytes {
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        store: &Store,
    ) -> Self::Fut {
        let body = std::mem::replace(req.body_mut(), Body::empty());

        FutureObj::new(Box::new(
            async move {
                let body = await!(body.into_vec()).map_err(mk_err)?;
                Ok(Bytes(body))
            },
        ))
    }
}

impl Deref for Bytes {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}
