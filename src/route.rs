use crate::{router::Router, Endpoint};

/// A handle to a route.
///
/// All HTTP requests are made against resources. After using [`App::at`] (or
/// [`Route::at`]) to establish a route, the `Route` type can be used to
/// establish endpoints for various HTTP methods at that path. Also, using
/// `nest`, it can be used to set up a subrouter.
pub struct Route<'a, AppData> {
    router: &'a mut Router<AppData>,
    path: String,
}

impl<'a, AppData: 'static> Route<'a, AppData> {
    pub(crate) fn new(router: &'a mut Router<AppData>, path: String) -> Route<'a, AppData> {
        Route { router, path }
    }

    /// Extend the route with the given `path`.
    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b, AppData> {
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
        }
    }

    pub fn nest(&mut self, f: impl FnOnce(&mut Route<'a, AppData>)) -> &mut Self {
        f(self);
        self
    }

    /// Add an endpoint for the given HTTP method
    pub fn method(&mut self, method: http::Method, ep: impl Endpoint<AppData>) -> &mut Self {
        self.router.add(&self.path, method, ep);
        self
    }

    /// Add an endpoint for `GET` requests
    pub fn get(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::GET, ep);
        self
    }

    /// Add an endpoint for `HEAD` requests
    pub fn head(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::HEAD, ep);
        self
    }

    /// Add an endpoint for `PUT` requests
    pub fn put(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::PUT, ep);
        self
    }

    /// Add an endpoint for `POST` requests
    pub fn post(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::POST, ep);
        self
    }

    /// Add an endpoint for `DELETE` requests
    pub fn delete(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::DELETE, ep);
        self
    }

    /// Add an endpoint for `OPTIONS` requests
    pub fn options(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::OPTIONS, ep);
        self
    }

    /// Add an endpoint for `CONNECT` requests
    pub fn connect(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::CONNECT, ep);
        self
    }

    /// Add an endpoint for `PATCH` requests
    pub fn patch(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::PATCH, ep);
        self
    }

    /// Add an endpoint for `TRACE` requests
    pub fn trace(&mut self, ep: impl Endpoint<AppData>) -> &mut Self {
        self.method(http::Method::TRACE, ep);
        self
    }
}
