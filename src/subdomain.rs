use crate::{log, namespace::Namespace, router::Router, router::Selection, Middleware, Route};
use std::sync::Arc;

/// Filter route on the subdomain the user registers
///
/// This middleware is a helper function that router uses to give users
/// access to route on subdomains on certain routes. If routes include
/// subdomains that can be parameters then it attaches it to the request
/// object.
///
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

    pub fn at<'b>(&'b mut self, path: &str) -> Route<'b, State> {
        Route::new(&mut self.router, path.to_owned())
    }
}
