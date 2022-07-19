use routefinder::{Captures, Router as MethodRouter};
use std::collections::HashMap;

use crate::{Next, Request, Response, StatusCode};

/// The routing table used by `Server`
///
/// Internally, we have a separate state machine per http method; indexing
/// by the method first allows the table itself to be more efficient.
#[allow(missing_debug_implementations)]
pub(crate) struct Router {
    method_map: HashMap<http_types::Method, MethodRouter<Next>>,
    all_method_router: MethodRouter<Next>,
}

impl std::fmt::Debug for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Router")
            .field("method_map", &self.method_map)
            .field("all_method_router", &self.all_method_router)
            .finish()
    }
}

/// The result of routing a URL
pub(crate) struct Selection {
    pub(crate) next: Next,
    pub(crate) params: Captures<'static, 'static>,
}

impl Router {
    pub(crate) fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            all_method_router: MethodRouter::new(),
        }
    }

    pub(crate) fn add(&mut self, path: &str, method: http_types::Method, ep: Next) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, ep)
            .unwrap()
    }

    pub(crate) fn add_all(&mut self, path: &str, ep: Next) {
        self.all_method_router.add(path, ep).unwrap()
    }

    pub(crate) fn route(&self, path: &str, method: http_types::Method) -> Selection {
        if let Some(m) = self
            .method_map
            .get(&method)
            .and_then(|r| r.best_match(path))
        {
            Selection {
                next: m.handler().clone(),
                params: m.captures().into_owned(),
            }
        } else if let Some(m) = self.all_method_router.best_match(path) {
            Selection {
                next: m.handler().clone(),
                params: m.captures().into_owned(),
            }
        } else if method == http_types::Method::Head {
            // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
            // if not then fallback to the behavior of HTTP GET else proceed as usual

            self.route(path, http_types::Method::Get)
        } else if self
            .method_map
            .iter()
            .filter(|(k, _)| **k != method)
            .any(|(_, r)| r.best_match(path).is_some())
        {
            // If this `path` can be handled by a callback registered with a different HTTP method
            // should return 405 Method Not Allowed
            Selection {
                next: Next::from_endpoint(method_not_allowed),
                params: Captures::default(),
            }
        } else {
            Selection {
                next: Next::from_endpoint(not_found_endpoint),
                params: Captures::default(),
            }
        }
    }
}

async fn not_found_endpoint(_req: Request) -> crate::Result {
    Ok(Response::new(StatusCode::NotFound))
}

async fn method_not_allowed(_req: Request) -> crate::Result {
    Ok(Response::new(StatusCode::MethodNotAllowed))
}
