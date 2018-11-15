use std::collections::HashMap;

use crate::{
    endpoint::{BoxedEndpoint, Endpoint},
    url_table::{ResolveResult, RouteMatch, UrlTable},
    Middleware,
};

/// TODO: Write documentation for `Router`
pub struct Router<Data> {
    table: UrlTable<Resource<Data>>,
    middleware: Vec<Box<dyn Middleware<Data> + Send + Sync>>,
}

impl<Data> Router<Data> {
    pub(crate) fn new() -> Router<Data> {
        Router {
            table: UrlTable::new(),
            middleware: vec![],
        }
    }

    /// Add a new resource at `path`.
    pub fn at<'a>(&'a mut self, path: &'a str) -> &mut Resource<Data> {
        self.table.setup(path)
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
    ) -> Option<(
        &'a BoxedEndpoint<Data>,
        RouteMatch<'a>,
        Vec<&'a (Middleware<Data> + Send + Sync)>
    )>
    {
        let mut table = &self.table;
        let mut params = Vec::new();
        let mut param_map = HashMap::new();
        let mut middlewares: Vec<_> = self.middleware.iter().map(|x| &**x).collect();

        for segment in path.split('/') {
            let result = table.resolve_segment(segment)?;
            match result {
                ResolveResult::Segment(next_table) => {
                    table = next_table;
                }
                ResolveResult::Wildcard { name, table: next_table } => {
                    params.push(segment);

                    if !name.is_empty() {
                        param_map.insert(name, segment);
                    }

                    table = next_table;
                }
            }

            // If the resource is a router, enter it
            if let Some(Resource(ResourceKind::Nested(router))) = table.root() {
                table = &router.table;
                middlewares.extend(router.middleware.iter().map(|x| &**x));
            }
        }

        let resource = table.root()?;
        let route_match = RouteMatch {
            vec: params,
            map: param_map,
        };

        let endpoints = match &resource.0 {
            ResourceKind::Empty => return None,
            ResourceKind::Endpoint(endpoints) => endpoints,
            ResourceKind::Nested(router) => unreachable!("Router::route should enter subroutes eagerly"),
        };

        // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
        // if not then fallback to the behavior of HTTP GET else proceed as usual
        let endpoint = if method == http::Method::HEAD && !endpoints.contains_key(&http::Method::HEAD) {
            endpoints.get(&http::Method::GET)
        } else {
            endpoints.get(method)
        }?;

        Some((endpoint, route_match, middlewares))
    }
}

/// A resource (identified by a URL).
///
/// All HTTP requests are made against resources. After using `App::at` to establish a resource path,
/// the `Resource` type can be used to establish endpoints for various HTTP methods at that path.
pub struct Resource<Data>(ResourceKind<Data>);

impl<Data> Default for Resource<Data> {
    fn default() -> Self {
        Resource(ResourceKind::default())
    }
}

enum ResourceKind<Data> {
    Empty,
    Endpoint(HashMap<http::Method, BoxedEndpoint<Data>>),
    Nested(Box<Router<Data>>),
}

impl<Data> Default for ResourceKind<Data> {
    fn default() -> Self {
        ResourceKind::Empty
    }
}

impl<Data> Resource<Data> {
    /// Nest a router in this path.
    pub fn nest<F>(&mut self, builder: F) where F: FnOnce(&mut Router<Data>) {
        if self.is_nested() {
            panic!("This path already has a router mounted");
        }
        if !self.is_empty() {
            panic!("This path already has endpoints");
        }

        let mut router = Router::new();
        builder(&mut router);
        self.0 = ResourceKind::Nested(Box::new(router));
    }

    /// Get whether this path has a router mounted.
    pub fn is_nested(&self) -> bool {
        match &self.0 {
            ResourceKind::Nested(_) => true,
            _ => false,
        }
    }

    /// Get whether this path is empty.
    pub fn is_empty(&self) -> bool {
        match self.0 {
            ResourceKind::Empty => true,
            _ => false,
        }
    }

    /// Add an endpoint for the given HTTP method
    pub fn method<T: Endpoint<Data, U>, U>(&mut self, method: http::Method, ep: T) {
        if let ResourceKind::Empty = self.0 {
            self.0 = ResourceKind::Endpoint(HashMap::new());
        }
        match &mut self.0 {
            ResourceKind::Endpoint(endpoints) => {
                if endpoints.contains_key(&method) {
                    panic!("A {} endpoint already exists for this path", method)
                }

                endpoints.insert(method, BoxedEndpoint::new(ep));
            }
            ResourceKind::Nested(router) => {
                panic!("This path has a router mounted.");
            }
            _ => {
                unreachable!();
            }
        }
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
