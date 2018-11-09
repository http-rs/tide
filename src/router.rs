use std::collections::HashMap;

use crate::{
    endpoint::{BoxedEndpoint, Endpoint},
    url_table::{RouteMatch, UrlTable},
};

pub(crate) struct Router<Data> {
    table: UrlTable<Resource<Data>>,
}

impl<Data> Router<Data> {
    pub(crate) fn new() -> Router<Data> {
        Router {
            table: UrlTable::new(),
        }
    }

    pub(crate) fn at<'a>(&'a mut self, path: &'a str) -> &mut Resource<Data> {
        self.table.setup(path)
    }

    pub(crate) fn route<'a>(
        &'a self,
        path: &'a str,
        method: &http::Method,
    ) -> Option<(&'a BoxedEndpoint<Data>, RouteMatch<'a>)> {
        // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
        // if not then fallback to the behavior of HTTP GET else proceed as usual
            if method == http::Method::HEAD && 
            !self.table.route(path).unwrap().0.endpoints.contains_key(&http::Method::HEAD){
               self.table
                   .route(path)
                   .and_then(|(r, p)| Some((r.endpoints.get(&http::Method::GET)?, p)))
            } else {            
               self.table
                   .route(path)
                   .and_then(|(r, p)| Some((r.endpoints.get(method)?, p)))
            }
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
        let old_ep = self
            .endpoints
            .insert(method.clone(), BoxedEndpoint::new(ep));

        if old_ep.is_some() {
            panic!("A {} endpoint already exists for this path", method)
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
