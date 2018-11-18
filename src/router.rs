use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    endpoint::{BoxedEndpoint, Endpoint},
    url_table::{RouteMatch, UrlTable},
    Middleware,
};

/// TODO: Document `Router`
pub struct Router<Data> {
    table: UrlTable<Resource<Data>>,
    middleware: Vec<Arc<dyn Middleware<Data> + Send + Sync>>,
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
    pub(crate) fn new() -> Router<Data> {
        Router {
            table: UrlTable::new(),
            middleware: Vec::new(),
        }
    }

    /// Add a new resource at `path`, relative to this router.
    ///
    /// Middlewares added before will be applied to the resource.
    pub fn at<'a>(&'a mut self, path: &'a str) -> ResourceHandle<'a, Data> {
        let table = self.table.setup_table(path);
        let middleware = &*self.middleware;
        ResourceHandle { table, middleware }
    }

    /// Add `middleware` to this router.
    pub fn middleware(&mut self, middleware: impl Middleware<Data> + 'static) -> &mut Self {
        self.middleware.push(Arc::new(middleware));
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
        let middleware = &*route.middleware;

        Some(RouteResult {
            endpoint,
            params: route_match,
            middleware,
        })
    }
}

/// A struct representing specific path of a router.
///
/// The struct implements `Deref` and `DerefMut` into `Resource`. You can use this to add a
/// resource to the path.
///
/// Also, using `nest`, you can set up a subrouter for the path.
pub struct ResourceHandle<'a, Data> {
    table: &'a mut UrlTable<Resource<Data>>,
    middleware: &'a [Arc<dyn Middleware<Data> + Send + Sync>],
}

impl<'a, Data> ResourceHandle<'a, Data> {
    /// "Nest" a subrouter to the path.
    pub fn nest<F>(self, builder: F)
    where
        F: FnOnce(&mut Router<Data>),
    {
        if self.table.resource().is_some() {
            panic!("This path already has a resource");
        }

        let mut subrouter = Router {
            table: UrlTable::new(),
            middleware: self.middleware.to_vec(),
        };
        builder(&mut subrouter);
        std::mem::swap(self.table, &mut subrouter.table);
    }
}

impl<'a, Data> std::ops::Deref for ResourceHandle<'a, Data> {
    type Target = Resource<Data>;

    fn deref(&self) -> &Resource<Data> {
        self.table
            .resource()
            .expect("Resource of this path has not been initialized.")
    }
}

impl<'a, Data> std::ops::DerefMut for ResourceHandle<'a, Data> {
    fn deref_mut(&mut self) -> &mut Resource<Data> {
        let resource = self.table.resource_mut();
        if resource.is_none() {
            let new_resource = Resource {
                endpoints: HashMap::new(),
                middleware: self.middleware.to_vec(),
            };
            *resource = Some(new_resource);
        }

        resource.as_mut().unwrap()
    }
}

/// A resource (identified by a URL).
///
/// All HTTP requests are made against resources. After using `App::at` to establish a resource path,
/// the `Resource` type can be used to establish endpoints for various HTTP methods at that path.
pub struct Resource<Data> {
    endpoints: HashMap<http::Method, BoxedEndpoint<Data>>,
    middleware: Vec<Arc<dyn Middleware<Data> + Send + Sync>>,
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
