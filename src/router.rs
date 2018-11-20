use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    endpoint::{BoxedEndpoint, Endpoint},
    url_table::{RouteMatch, UrlTable},
    Middleware,
};

/// TODO: Document `Router`
pub struct Router<Data> {
    idx: usize,
    table: UrlTable<ResourceData<Data>>,
    middleware_list: Vec<Vec<Arc<dyn Middleware<Data> + Send + Sync>>>,
}

pub(crate) struct RouteResult<'a, Data> {
    pub(crate) endpoint: &'a BoxedEndpoint<Data>,
    pub(crate) params: RouteMatch<'a>,
    pub(crate) middleware: &'a [Arc<dyn Middleware<Data> + Send + Sync>],
}

impl<Data> Default for Router<Data> {
    fn default() -> Router<Data> {
        Router::new()
    }
}

impl<Data> Router<Data> {
    /// Create a new top-level router.
    pub(crate) fn new() -> Router<Data> {
        Router {
            idx: 0,
            table: UrlTable::new(),
            middleware_list: vec![Vec::new()],
        }
    }

    /// Add a new resource at `path`, relative to this router.
    ///
    /// Middlewares added before will be applied to the resource.
    pub fn at<'a>(&'a mut self, path: &'a str) -> Resource<'a, Data> {
        let table = self.table.setup_table(path);
        let next_router_idx = self.idx + self.middleware_list.len();
        Resource {
            router_idx: self.idx,
            next_router_idx,
            table,
            middleware_list: &mut self.middleware_list,
        }
    }

    /// Add `middleware` to this router.
    pub fn middleware(&mut self, middleware: impl Middleware<Data> + 'static) -> &mut Self {
        let middleware = Arc::new(middleware);
        for middleware_list_item in self.middleware_list.iter_mut() {
            middleware_list_item.push(middleware.clone());
        }
        self
    }

    pub(crate) fn route<'a>(
        &'a self,
        path: &'a str,
        method: &http::Method,
    ) -> Option<RouteResult<'a, Data>> {
        let (route, route_match) = self.table.route(path)?;
        // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
        // if not then fallback to the behavior of HTTP GET else proceed as usual
        let endpoint =
            if method == http::Method::HEAD && !route.endpoints.contains_key(&http::Method::HEAD) {
                route.endpoints.get(&http::Method::GET)?
            } else {
                route.endpoints.get(method)?
            };
        let middleware = &*self.middleware_list[route.router_idx];

        Some(RouteResult {
            endpoint,
            params: route_match,
            middleware,
        })
    }
}

/// A handle to a resource (identified by a URL).
///
/// All HTTP requests are made against resources. After using `App::at` to establish a resource path,
/// the `Resource` type can be used to establish endpoints for various HTTP methods at that path.
///
/// Also, the `Resource` type can be used to set up a subrouter using `nest`.
pub struct Resource<'a, Data> {
    router_idx: usize,
    next_router_idx: usize,
    table: &'a mut UrlTable<ResourceData<Data>>,
    middleware_list: &'a mut Vec<Vec<Arc<dyn Middleware<Data> + Send + Sync>>>,
}

struct ResourceData<Data> {
    endpoints: HashMap<http::Method, BoxedEndpoint<Data>>,
    router_idx: usize,
}

impl<'a, Data> Resource<'a, Data> {
    /// "Nest" a subrouter to the path.
    ///
    /// If resources are already present at current path and its descendents, they will be discarded.
    pub fn nest<F>(self, builder: F)
    where
        F: FnOnce(&mut Router<Data>),
    {
        let mut subrouter = Router {
            idx: self.next_router_idx,
            table: UrlTable::new(),
            middleware_list: vec![self.middleware_list[0].clone()],
        };
        builder(&mut subrouter);
        *self.table = subrouter.table;
        self.middleware_list.extend(subrouter.middleware_list);
    }

    /// Add an endpoint for the given HTTP method
    pub fn method<T: Endpoint<Data, U>, U>(&mut self, method: http::Method, ep: T) {
        let resource = self.table.resource_mut();
        if resource.is_none() {
            let new_resource = ResourceData {
                endpoints: HashMap::new(),
                router_idx: self.router_idx,
            };
            *resource = Some(new_resource);
        }
        let resource = resource.as_mut().unwrap();

        if resource.endpoints.contains_key(&method) {
            panic!("A {} endpoint already exists for this path", method)
        }

        resource.endpoints.insert(method, BoxedEndpoint::new(ep));
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
