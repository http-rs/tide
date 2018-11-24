use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    endpoint::{BoxedEndpoint, Endpoint},
    url_table::{RouteMatch, UrlTable},
    Middleware,
};

/// A core type for routing.
///
/// The `Router` type can be used to set up routes and resources, and to apply middleware.
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

    /// Apply `middleware` to this router.
    ///
    /// Note that the order of nesting subrouters and applying middleware matters. If there are
    /// nested subrouters *before* the method call, the given middleware will be applied *after*
    /// the subrouter middleware.
    ///
    /// ```
    /// # #![feature(futures_api, async_await)]
    /// # struct A;
    /// # impl tide::Middleware<()> for A {}
    /// # struct B;
    /// # impl tide::Middleware<()> for B {}
    /// # let mut app = tide::App::new(());
    /// # let router = app.router();
    /// router.at("a1").nest(|router| {
    ///     router.middleware(A);
    ///     router.at("").get(async || "A then B");
    /// });
    /// router.middleware(B);
    /// router.at("a2").nest(|router| {
    ///     router.middleware(A);
    ///     router.at("").get(async || "B then A");
    /// });
    /// ```
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
/// All HTTP requests are made against resources. After using `Router::at` (or `App::at`) to
/// establish a resource path, the `Resource` type can be used to establish endpoints for various
/// HTTP methods at that path. Also, using `nest`, it can be used to set up a subrouter.
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
    /// This method will build a fresh `Router` and give a mutable reference to it to the builder
    /// function. Builder can set up a subrouter using the `Router`. All middleware applied inside
    /// the builder will be local to the subrouter and its descendents.
    ///
    /// If resources are already present, they will be discarded.
    pub fn nest(self, builder: impl FnOnce(&mut Router<Data>)) {
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

    /// Add an endpoint for `OPTIONS` requests
    pub fn options<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::OPTIONS, ep)
    }

    /// Add an endpoint for `CONNECT` requests
    pub fn connect<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::CONNECT, ep)
    }

    /// Add an endpoint for `PATCH` requests
    pub fn patch<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::PATCH, ep)
    }

    /// Add an endpoint for `TRACE` requests
    pub fn trace<T: Endpoint<Data, U>, U>(&mut self, ep: T) {
        self.method(http::Method::TRACE, ep)
    }
}

#[cfg(test)]
mod tests {
    use futures::{executor::block_on, future::FutureObj};

    use super::*;
    use crate::{body::Body, AppData, Request, Response};

    async fn simulate_request<'a, Data: Default + Clone>(
        router: &'a Router<Data>,
        path: &'a str,
        method: &'a http::Method,
    ) -> Option<Response> {
        let RouteResult {
            endpoint,
            params,
            middleware,
        } = router.route(path, method)?;

        let mut data = Data::default();
        let mut req = http::Request::builder()
            .method(method)
            .body(Body::empty())
            .unwrap();
        for m in middleware.iter() {
            if let Err(resp) = await!(m.request(&mut data, &mut req, &params)) {
                return Some(resp.map(Into::into));
            }
        }

        let (head, mut resp) = await!(endpoint.call(data.clone(), req, params));

        for m in middleware.iter() {
            await!(m.response(&mut data, &head, &mut resp))
        }

        Some(resp.map(Into::into))
    }

    fn route_middleware_count<Data>(
        router: &Router<Data>,
        path: &str,
        method: &http::Method,
    ) -> Option<usize> {
        let route_result = router.route(path, method)?;
        Some(route_result.middleware.len())
    }

    #[test]
    fn simple_static() {
        let mut router: Router<()> = Router::new();
        router.at("/").get(async || "/");
        router.at("/foo").get(async || "/foo");
        router.at("/foo/bar").get(async || "/foo/bar");

        for path in &["/", "/foo", "/foo/bar"] {
            let res =
                if let Some(res) = block_on(simulate_request(&router, path, &http::Method::GET)) {
                    res
                } else {
                    panic!("Routing of path `{}` failed", path);
                };
            let body =
                block_on(res.into_body().read_to_vec()).expect("Reading body should succeed");
            assert_eq!(&*body, path.as_bytes());
        }
    }

    #[test]
    fn nested_static() {
        let mut router: Router<()> = Router::new();
        router.at("/a").get(async || "/a");
        router.at("/b").nest(|router| {
            router.at("/").get(async || "/b");
            router.at("/a").get(async || "/b/a");
            router.at("/b").get(async || "/b/b");
            router.at("/c").nest(|router| {
                router.at("/a").get(async || "/b/c/a");
                router.at("/b").get(async || "/b/c/b");
            });
            router.at("/d").get(async || "/b/d");
        });
        router.at("/a/a").nest(|router| {
            router.at("/a").get(async || "/a/a/a");
            router.at("/b").get(async || "/a/a/b");
        });
        router.at("/a/b").nest(|router| {
            router.at("/").get(async || "/a/b");
        });

        for failing_path in &["/", "/a/a", "/a/b/a"] {
            if block_on(simulate_request(&router, failing_path, &http::Method::GET)).is_some() {
                panic!(
                    "Routing of path `{}` should fail, but was successful",
                    failing_path
                );
            };
        }

        for path in &[
            "/a", "/a/a/a", "/a/a/b", "/a/b", "/b", "/b/a", "/b/b", "/b/c/a", "/b/c/b", "/b/d",
        ] {
            let res =
                if let Some(res) = block_on(simulate_request(&router, path, &http::Method::GET)) {
                    res
                } else {
                    panic!("Routing of path `{}` failed", path);
                };
            let body =
                block_on(res.into_body().read_to_vec()).expect("Reading body should succeed");
            assert_eq!(&*body, path.as_bytes());
        }
    }

    #[test]
    fn multiple_methods() {
        let mut router: Router<()> = Router::new();
        router
            .at("/a")
            .nest(|router| router.at("/b").get(async || "/a/b GET"));
        router.at("/a/b").post(async || "/a/b POST");

        for (path, method) in &[("/a/b", http::Method::GET), ("/a/b", http::Method::POST)] {
            let res = if let Some(res) = block_on(simulate_request(&router, path, &method)) {
                res
            } else {
                panic!("Routing of {} `{}` failed", method, path);
            };
            let body =
                block_on(res.into_body().read_to_vec()).expect("Reading body should succeed");
            assert_eq!(&*body, format!("{} {}", path, method).as_bytes());
        }
    }

    #[test]
    #[should_panic]
    fn duplicate_endpoint_fails() {
        let mut router: Router<()> = Router::new();
        router
            .at("/a")
            .nest(|router| router.at("/b").get(async || "")); // flattened into /a/b
        router.at("/a/b").get(async || "duplicate");
    }

    #[test]
    fn simple_middleware() {
        struct A;
        impl Middleware<()> for A {}

        let mut router: Router<()> = Router::new();
        router.middleware(A);
        router.at("/").get(async || "/");
        router.at("/b").nest(|router| {
            router.at("/").get(async || "/b");
            router.middleware(A);
        });

        assert_eq!(
            route_middleware_count(&router, "/", &http::Method::GET),
            Some(1)
        );
        assert_eq!(
            route_middleware_count(&router, "/b", &http::Method::GET),
            Some(2)
        );
    }

    #[test]
    fn middleware_apply_order() {
        #[derive(Default, Clone, Debug)]
        struct Data(Vec<usize>);
        struct Pusher(usize);
        impl Middleware<Data> for Pusher {
            fn request(
                &self,
                data: &mut Data,
                req: &mut Request,
                _: &RouteMatch<'_>,
            ) -> FutureObj<'static, Result<(), Response>> {
                data.0.push(self.0);
                FutureObj::new(Box::new(async { Ok(()) }))
            }
        }

        // The order of endpoint and middleware does not matter
        // The order of subrouter and middleware DOES matter
        let mut router: Router<Data> = Router::new();
        router.middleware(Pusher(0));
        router.at("/").get(async move |data: AppData<Data>| {
            if (data.0).0 == [0, 2] {
                http::StatusCode::OK
            } else {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
        });
        router.at("/a").nest(|router| {
            router.at("/").get(async move |data: AppData<Data>| {
                if (data.0).0 == [0, 1, 2] {
                    http::StatusCode::OK
                } else {
                    http::StatusCode::INTERNAL_SERVER_ERROR
                }
            });
            router.middleware(Pusher(1));
        });
        router.middleware(Pusher(2));
        router.at("/b").nest(|router| {
            router.at("/").get(async move |data: AppData<Data>| {
                if (data.0).0 == [0, 2, 1] {
                    http::StatusCode::OK
                } else {
                    http::StatusCode::INTERNAL_SERVER_ERROR
                }
            });
            router.middleware(Pusher(1));
        });

        for path in &["/", "/a", "/b"] {
            let res = block_on(simulate_request(&router, path, &http::Method::GET)).unwrap();
            assert_eq!(res.status(), 200);
        }
    }
}
