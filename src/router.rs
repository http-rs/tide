use std::collections::HashMap;

use crate::{
    endpoint::{BoxedEndpoint, Endpoint},
    url_table::{RouteMatch, UrlTable},
    Middleware,
};

pub use crate::url_table::ResourceHandle;

pub(crate) struct EndpointInfo<'a, Data> {
    pub(crate) endpoint: &'a BoxedEndpoint<Data>,
    pub(crate) params: RouteMatch<'a>,
    pub(crate) middleware: Vec<&'a (Middleware<Data> + Send + Sync)>,
}

/// TODO: Write documentation for `Router`
pub struct Router<Data> {
    table: UrlTable<Router<Data>>,
    middleware: Vec<Box<dyn Middleware<Data> + Send + Sync>>,
}

impl<Data> crate::url_table::Router for Router<Data> {
    type Resource = Resource<Data>;

    fn table(&self) -> &UrlTable<Self> {
        &self.table
    }

    fn table_mut(&mut self) -> &mut UrlTable<Self> {
        &mut self.table
    }
}

impl<Data> Default for Router<Data> {
    fn default() -> Router<Data> {
        Self::new()
    }
}

impl<Data> Router<Data> {
    pub fn new() -> Router<Data> {
        Router {
            table: UrlTable::new(),
            middleware: vec![],
        }
    }

    /// Add a new resource at `path`.
    pub fn at<'a>(&'a mut self, path: &'a str) -> ResourceHandle<'a, Self> {
        <Self as crate::url_table::Router>::at(self, path)
    }

    /// Apply `middleware` to this router.
    pub fn middleware(&mut self, middleware: impl Middleware<Data> + 'static) -> &mut Self {
        self.middleware.push(Box::new(middleware));
        self
    }

    pub(crate) fn route<'a>(
        &'a self,
        path: &'a str,
        method: &http::Method,
    ) -> Option<EndpointInfo<'a, Data>> {
        let route_result = <Self as crate::url_table::Router>::route(self, path)?;

        let resource = route_result.resource;
        let params = route_result.route_match;
        let middleware: Vec<_> = route_result
            .routers
            .into_iter()
            .flat_map(|router| router.middleware.iter().map(|m| &**m))
            .collect();

        let endpoints = &resource.endpoints;

        // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
        // if not then fallback to the behavior of HTTP GET else proceed as usual
        let endpoint =
            if method == http::Method::HEAD && !endpoints.contains_key(&http::Method::HEAD) {
                endpoints.get(&http::Method::GET)
            } else {
                endpoints.get(method)
            }?;

        Some(EndpointInfo {
            endpoint,
            params,
            middleware,
        })
    }
}

/// A resource (identified by a URL).
///
/// All HTTP requests are made against resources. After using `App::at` to establish a resource path,
/// the `Resource` type can be used to establish endpoints for various HTTP methods at that path.
pub struct Resource<Data> {
    endpoints: HashMap<http::Method, BoxedEndpoint<Data>>,
}

impl<Data> Default for Resource<Data> {
    fn default() -> Self {
        Resource {
            endpoints: HashMap::new(),
        }
    }
}

impl<Data> Resource<Data> {
    /// Add an endpoint for the given HTTP method
    pub fn method<T: Endpoint<Data, U>, U>(&mut self, method: http::Method, ep: T) {
        if self.endpoints.contains_key(&method) {
            panic!("A {} endpoint already exists for this path", method)
        }

        self.endpoints.insert(method, BoxedEndpoint::new(ep));
    }

    /// Add an endpoint for `GET` requests
    pub fn get<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::GET, ep)
    }

    /// Add an endpoint for `HEAD` requests
    pub fn head<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::HEAD, ep)
    }

    /// Add an endpoint for `PUT` requests
    pub fn put<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::PUT, ep)
    }

    /// Add an endpoint for `POST` requests
    pub fn post<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::POST, ep)
    }

    /// Add an endpoint for `DELETE` requests
    pub fn delete<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::DELETE, ep)
    }
}
