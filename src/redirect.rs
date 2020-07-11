//! HTTP redirection endpoint.
//!
//! # Examples
//!
//! ```no_run
//! # use async_std::task::block_on;
//! # fn main() -> Result<(), std::io::Error> { block_on(async {
//! #
//! use tide::Redirect;
//!
//! let mut app = tide::new();
//! app.at("/").get(|_| async { Ok("meow") });
//! app.at("/nori").get(Redirect::temporary("/"));
//! app.listen("127.0.0.1:8080").await?;
//! #
//! # Ok(()) }) }
//! ```

use crate::http::headers::LOCATION;
use crate::StatusCode;
use crate::{Endpoint, Request, Response};

/// A redirection endpoint.
///
/// # Example
///
/// ```
/// # use tide::{Response, Redirect, Request, StatusCode};
/// # fn next_product() -> Option<String> { None }
/// # #[allow(dead_code)]
/// async fn route_handler(request: Request<()>) -> tide::Result {
///     if let Some(product_url) = next_product() {
///         Ok(Redirect::new(product_url).into())
///     } else {
///         //...
/// #       Ok(Response::new(StatusCode::Ok)) //...
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Redirect<T: AsRef<str>> {
    status: StatusCode,
    location: T,
}

impl<T: AsRef<str>> Redirect<T> {
    /// Creates an endpoint that represents a redirect to `location`.
    ///
    /// Uses status code 302 Found.
    pub fn new(location: T) -> Self {
        Self {
            status: StatusCode::Found,
            location,
        }
    }

    /// Creates an endpoint that represents a permanent redirect to `location`.
    ///
    /// Uses status code 301 Permanent Redirect.
    pub fn permanent(location: T) -> Self {
        Self {
            status: StatusCode::PermanentRedirect,
            location,
        }
    }

    /// Creates an endpoint that represents a temporary redirect to `location`.
    ///
    /// Uses status code 307 Temporary Redirect.
    pub fn temporary(location: T) -> Self {
        Self {
            status: StatusCode::TemporaryRedirect,
            location,
        }
    }

    /// Creates an endpoint that represents a see other redirect to `location`.
    ///
    /// Uses status code 303 See Other.
    pub fn see_other(location: T) -> Self {
        Self {
            status: StatusCode::SeeOther,
            location,
        }
    }
}

#[async_trait::async_trait]
impl<State, T> Endpoint<State> for Redirect<T>
where
    State: Send + Sync + 'static,
    T: AsRef<str> + Send + Sync + 'static,
{
    async fn call(&self, _req: Request<State>) -> crate::Result<Response> {
        Ok(self.into())
    }
}

impl<T: AsRef<str>> Into<Response> for Redirect<T> {
    fn into(self) -> Response {
        (&self).into()
    }
}

impl<T: AsRef<str>> Into<Response> for &Redirect<T> {
    fn into(self) -> Response {
        let mut res = Response::new(self.status);
        res.insert_header(LOCATION, self.location.as_ref());
        res
    }
}
