use std::fmt::Debug;
use std::io;
use std::path::Path;
use std::sync::Arc;

use crate::endpoint::MiddlewareEndpoint;
use crate::fs::ServeDir;
use crate::log;
use crate::utils::BoxFuture;
use crate::{router::Router, Endpoint, Middleware};

/// A handle to a route.
///
/// All HTTP requests are made against resources. After using [`Server::at`] (or
/// [`Route::at`]) to establish a route, the `Route` type can be used to
/// establish endpoints for various HTTP methods at that path. Also, using
/// `nest`, it can be used to set up a subrouter.
///
/// [`Server::at`]: ./struct.Server.html#method.at
#[allow(missing_debug_implementations)]
pub struct Route<'a, State> {
    router: &'a mut Router<State>,
    path: String,
    middleware: Vec<Arc<dyn Middleware<State>>>,
    /// Indicates whether the path of current route is treated as a prefix. Set by
    /// [`strip_prefix`].
    ///
    /// [`strip_prefix`]: #method.strip_prefix
    prefix: bool,
}

impl<'a, State: Send + Sync + 'static> Route<'a, State> {
    pub(crate) fn new(router: &'a mut Router<State>, path: String) -> Route<'a, State> {
        Route {
            router,
            path,
            middleware: Vec::new(),
            prefix: false,
        }
    }

    /// Extend the route with the given `path`.
    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b, State> {
        let mut p = self.path.clone();

        if !p.ends_with('/') && !path.starts_with('/') {
            p.push_str("/");
        }

        if path != "/" {
            p.push_str(path);
        }

        Route {
            router: &mut self.router,
            path: p,
            middleware: self.middleware.clone(),
            prefix: false,
        }
    }

    /// Get the current path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Treat the current path as a prefix, and strip prefixes from requests.
    ///
    /// This method is marked unstable as its name might change in the near future.
    ///
    /// Endpoints will be given a path with the prefix removed.
    #[cfg(any(feature = "unstable", feature = "docs"))]
    #[cfg_attr(feature = "docs", doc(cfg(unstable)))]
    pub fn strip_prefix(&mut self) -> &mut Self {
        self.prefix = true;
        self
    }

    /// Apply the given middleware to the current route.
    pub fn middleware<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        log::trace!(
            "Adding middleware {} to route {:?}",
            middleware.name(),
            self.path
        );
        self.middleware.push(Arc::new(middleware));
        self
    }

    /// Reset the middleware chain for the current route, if any.
    pub fn reset_middleware(&mut self) -> &mut Self {
        self.middleware.clear();
        self
    }

    /// Nest a [`Server`] at the current path.
    ///
    /// [`Server`]: struct.Server.html
    pub fn nest<InnerState>(&mut self, service: crate::Server<InnerState>) -> &mut Self
    where
        State: Send + Sync + 'static,
        InnerState: Send + Sync + 'static,
    {
        self.prefix = true;
        self.all(service);
        self.prefix = false;
        self
    }

    /// Serve a directory statically.
    ///
    /// Each file will be streamed from disk, and a mime type will be determined
    /// based on magic bytes.
    ///
    /// # Examples
    ///
    /// Serve the contents of the local directory `./public/images/*` from
    /// `localhost:8080/images/*`.
    ///
    /// ```no_run
    /// #[async_std::main]
    /// async fn main() -> Result<(), std::io::Error> {
    ///     let mut app = tide::new();
    ///     app.at("/public/images").serve_dir("images/")?;
    ///     app.listen("127.0.0.1:8080").await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn serve_dir(&mut self, dir: impl AsRef<Path>) -> io::Result<()> {
        // Verify path exists, return error if it doesn't.
        let dir = dir.as_ref().to_owned().canonicalize()?;
        let prefix = self.path().to_string();
        self.at("*").get(ServeDir::new(prefix, dir));
        Ok(())
    }

    /// Add an endpoint for the given HTTP method
    pub fn method(&mut self, method: http_types::Method, ep: impl Endpoint<State>) -> &mut Self {
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);
            let (ep1, ep2): (Box<dyn Endpoint<_>>, Box<dyn Endpoint<_>>) =
                if self.middleware.is_empty() {
                    let ep = Box::new(ep);
                    (ep.clone(), ep)
                } else {
                    let ep = Box::new(MiddlewareEndpoint::wrap_with_middleware(
                        ep,
                        &self.middleware,
                    ));
                    (ep.clone(), ep)
                };
            self.router.add(&self.path, method, ep1);
            let wildcard = self.at("*--tide-path-rest");
            wildcard.router.add(&wildcard.path, method, ep2);
        } else {
            let ep: Box<dyn Endpoint<_>> = if self.middleware.is_empty() {
                Box::new(ep)
            } else {
                Box::new(MiddlewareEndpoint::wrap_with_middleware(
                    ep,
                    &self.middleware,
                ))
            };
            self.router.add(&self.path, method, ep);
        }
        self
    }

    /// Add an endpoint for all HTTP methods, as a fallback.
    ///
    /// Routes with specific HTTP methods will be tried first.
    pub fn all(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);
            let (ep1, ep2): (Box<dyn Endpoint<_>>, Box<dyn Endpoint<_>>) =
                if self.middleware.is_empty() {
                    let ep = Box::new(ep);
                    (ep.clone(), ep)
                } else {
                    let ep = Box::new(MiddlewareEndpoint::wrap_with_middleware(
                        ep,
                        &self.middleware,
                    ));
                    (ep.clone(), ep)
                };
            self.router.add_all(&self.path, ep1);
            let wildcard = self.at("*--tide-path-rest");
            wildcard.router.add_all(&wildcard.path, ep2);
        } else {
            let ep: Box<dyn Endpoint<_>> = if self.middleware.is_empty() {
                Box::new(ep)
            } else {
                Box::new(MiddlewareEndpoint::wrap_with_middleware(
                    ep,
                    &self.middleware,
                ))
            };
            self.router.add_all(&self.path, ep);
        }
        self
    }

    /// Add an endpoint for `GET` requests
    pub fn get(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Get, ep);
        self
    }

    /// Add an endpoint for `HEAD` requests
    pub fn head(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Head, ep);
        self
    }

    /// Add an endpoint for `PUT` requests
    pub fn put(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Put, ep);
        self
    }

    /// Add an endpoint for `POST` requests
    pub fn post(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Post, ep);
        self
    }

    /// Add an endpoint for `DELETE` requests
    pub fn delete(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Delete, ep);
        self
    }

    /// Add an endpoint for `OPTIONS` requests
    pub fn options(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Options, ep);
        self
    }

    /// Add an endpoint for `CONNECT` requests
    pub fn connect(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Connect, ep);
        self
    }

    /// Add an endpoint for `PATCH` requests
    pub fn patch(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Patch, ep);
        self
    }

    /// Add an endpoint for `TRACE` requests
    pub fn trace(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http_types::Method::Trace, ep);
        self
    }
}

#[derive(Debug)]
struct StripPrefixEndpoint<E>(std::sync::Arc<E>);

impl<E> StripPrefixEndpoint<E> {
    fn new(ep: E) -> Self {
        Self(std::sync::Arc::new(ep))
    }
}

impl<E> Clone for StripPrefixEndpoint<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<State, E> Endpoint<State> for StripPrefixEndpoint<E>
where
    State: Send + Sync + 'static,
    E: Endpoint<State>,
{
    fn call<'a>(&'a self, req: crate::Request, _: State) -> BoxFuture<'a, crate::Result> {
        let crate::Request {
            mut req,
            route_params,
        } = req;

        let rest = crate::request::rest(&route_params).unwrap_or_else(|| "");
        req.url_mut().set_path(&rest);

        self.0.call(crate::Request {
            req,
            route_params,
        })
    }
}
