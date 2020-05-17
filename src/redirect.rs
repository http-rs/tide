//! HTTP redirection endpoint.
//!
//! # Examples
//!
//! ```no_run
//! # use futures::executor::block_on;
//! # fn main() -> Result<(), std::io::Error> { block_on(async {
//! #
//! use tide::Redirect;
//!
//! let mut app = tide::new();
//! app.at("/").get(|_| async move { Ok("meow") });
//! app.at("/nori").get(Redirect::temporary("/"));
//! app.listen("127.0.0.1:8080").await?;
//! #
//! # Ok(()) }) }
//! ```

use crate::utils::BoxFuture;
use crate::StatusCode;
use crate::{Endpoint, Request, Response};

/// A redirection endpoint.
#[derive(Debug, Clone)]
pub struct Redirect<T: AsRef<str>> {
    status: StatusCode,
    location: T,
}

impl<T: AsRef<str>> Redirect<T> {
    /// Creates an endpoint that represents a permanent redirect to `location`.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use tide::{Response, Redirect, Request, StatusCode};
    /// # fn canonicalize(uri: &url::Url) -> Option<&url::Url> { None }
    /// # #[allow(dead_code)]
    /// async fn route_handler(request: Request<()>) -> tide::Result {
    ///     if let Some(canonical_redirect) = canonicalize(request.uri()) {
    ///         Ok(Redirect::permanent(canonical_redirect).into())
    ///     } else {
    ///          //...
    /// #        Ok(Response::new(StatusCode::Ok)) // ...
    ///     }
    /// }
    /// ```
    pub fn permanent(location: T) -> Self {
        Self {
            status: StatusCode::PermanentRedirect,
            location,
        }
    }

    /// Creates an endpoint that represents a temporary redirect to `location`.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use tide::{Response, Redirect, Request, StatusCode};
    /// # fn special_sale_today() -> Option<String> { None }
    /// # #[allow(dead_code)]
    /// async fn route_handler(request: Request<()>) -> tide::Result {
    ///     if let Some(sale_url) = special_sale_today() {
    ///         Ok(Redirect::temporary(sale_url).into())
    ///     } else {
    ///         //...
    /// #       Ok(Response::new(StatusCode::Ok)) //...
    ///     }
    /// }
    /// ```
    pub fn temporary(location: T) -> Self {
        Self {
            status: StatusCode::TemporaryRedirect,
            location,
        }
    }

    /// Creates an endpoint that represents a see other redirect to `location`.
    ///
    /// GET methods are unchanged.
    /// Other methods are changed to GET and their body lost.
    ///
    /// # Example
    ///
    /// ```
    /// # use tide::{Response, Redirect, Request, StatusCode};
    /// # fn next_product() -> Option<String> { None }
    /// # #[allow(dead_code)]
    /// async fn route_handler(request: Request<()>) -> tide::Result {
    ///     if let Some(product_url) = next_product() {
    ///         Ok(Redirect::see_other(product_url).into())
    ///     } else {
    ///         //...
    /// #       Ok(Response::new(StatusCode::Ok)) //...
    ///     }
    /// }
    /// ```
    pub fn see_other(location: T) -> Self {
        Self {
            status: StatusCode::SeeOther,
            location,
        }
    }

    /// Returns response with equivalent redirect.
    pub fn response(&self) -> Response {
        Response::new(self.status).set_header("location".parse().unwrap(), &self.location)
    }
}

impl<State, T> Endpoint<State> for Redirect<T>
where
    T: AsRef<str> + Send + Sync + 'static,
{
    fn call<'a>(&'a self, _req: Request<State>) -> BoxFuture<'a, crate::Result<Response>> {
        let res = self.response();
        Box::pin(async move { Ok(res) })
    }
}

impl<T: AsRef<str>> Into<Response> for Redirect<T> {
    fn into(self) -> Response {
        self.response()
    }
}
