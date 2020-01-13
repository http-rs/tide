use crate::{router::Router, Endpoint};

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
    /// Indicates whether the path of current route is treated as a prefix. Set by
    /// [`strip_prefix`].
    ///
    /// [`strip_prefix`]: #method.strip_prefix
    prefix: bool,
}

impl<'a, State: 'static> Route<'a, State> {
    pub(crate) fn new(router: &'a mut Router<State>, path: String) -> Route<'a, State> {
        Route {
            router,
            path,
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
            prefix: false,
        }
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

    /// Nest a [`Server`] at the current path.
    ///
    /// [`Server`]: struct.Server.html
    pub fn nest<IState>(&mut self, service: crate::Server<IState>) -> &mut Self
    where
        State: Send + Sync + 'static,
        IState: Send + Sync + 'static,
    {
        self.prefix = true;
        self.all(service.into_http_service());
        self.prefix = false;
        self
    }

    /// Add an endpoint for the given HTTP method
    pub fn method(&mut self, method: http::Method, ep: impl Endpoint<State>) -> &mut Self {
        if self.prefix {
            let ep = StripPrefixEndpoint::new(ep);
            self.router.add(&self.path, method.clone(), ep.clone());
            let wildcard = self.at("*--tide-path-rest");
            wildcard.router.add(&wildcard.path, method, ep);
        } else {
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
            self.router.add_all(&self.path, ep.clone());
            let wildcard = self.at("*--tide-path-rest");
            wildcard.router.add_all(&wildcard.path, ep);
        } else {
            self.router.add_all(&self.path, ep);
        }
        self
    }

    /// Add an endpoint for `GET` requests
    pub fn get(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::GET, ep);
        self
    }

    /// Add an endpoint for `HEAD` requests
    pub fn head(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::HEAD, ep);
        self
    }

    /// Add an endpoint for `PUT` requests
    pub fn put(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::PUT, ep);
        self
    }

    /// Add an endpoint for `POST` requests
    pub fn post(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::POST, ep);
        self
    }

    /// Add an endpoint for `DELETE` requests
    pub fn delete(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::DELETE, ep);
        self
    }

    /// Add an endpoint for `OPTIONS` requests
    pub fn options(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::OPTIONS, ep);
        self
    }

    /// Add an endpoint for `CONNECT` requests
    pub fn connect(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::CONNECT, ep);
        self
    }

    /// Add an endpoint for `PATCH` requests
    pub fn patch(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::PATCH, ep);
        self
    }

    /// Add an endpoint for `TRACE` requests
    pub fn trace(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::TRACE, ep);
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

impl<State, E: Endpoint<State>> Endpoint<State> for StripPrefixEndpoint<E> {
    type Fut = E::Fut;

    fn call(&self, mut req: crate::Request<State>) -> Self::Fut {
        let rest = req.rest().unwrap_or("");
        let mut path_and_query = format!("/{}", rest);
        let uri = req.uri();
        if let Some(query) = uri.query() {
            path_and_query.push('?');
            path_and_query.push_str(query);
        }
        let mut new_uri = http::Uri::builder();
        if let Some(scheme) = uri.scheme_part() {
            new_uri.scheme(scheme.clone());
        }
        if let Some(authority) = uri.authority_part() {
            new_uri.authority(authority.clone());
        }
        new_uri.path_and_query(path_and_query.as_str());
        let new_uri = new_uri.build().unwrap();
        *req.request.uri_mut() = new_uri;

        self.0.call(req)
    }
}
