use crate::{log, namespace::Namespace, router::Router, router::Selection, Middleware, Route};
use std::sync::Arc;

/// A handle to a subdomain
///
/// All routes can be nested inside of a subdomain using [`Server::subdomain`]
/// to establish a subdomain. The `Subdomain` type can be used to establish
/// various `Route`.
///
/// [`Server::subdomain`]: ./struct.Server.html#method.subdomain
#[allow(missing_debug_implementations)]
pub struct Subdomain<State> {
    subdomain: String,
    router: Router<State>,
    middleware: Vec<Arc<dyn Middleware<State>>>,
}

impl<State: Clone + Send + Sync + 'static> Subdomain<State> {
    pub(crate) fn new<'a>(
        namespace: &'a mut Namespace<State>,
        subdomain: &str,
    ) -> &'a mut Subdomain<State> {
        let router = Self {
            subdomain: subdomain.to_owned(),
            router: Router::new(),
            middleware: Vec::new(),
        };
        namespace.add(router.subdomain.clone(), router)
    }

    pub(crate) fn route<'a>(&self, path: &str, method: http_types::Method) -> Selection<'_, State> {
        self.router.route(path, method)
    }

    pub(crate) fn middleware(&self) -> &Vec<Arc<dyn Middleware<State>>> {
        &self.middleware
    }

    /// Create a route on the given subdomain
    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b, State> {
        Route::new(&mut self.router, path.to_owned())
    }

    /// Apply the given middleware to the current route
    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        log::trace!(
            "Adding middleware {} to subdomain {:?}",
            middleware.name(),
            self.subdomain
        );
        self.middleware.push(Arc::new(middleware));
        self
    }
}
