use routefinder::{Captures, Router as MethodRouter};
use std::collections::HashMap;
use std::sync::Arc;

use crate::endpoint::DynEndpoint;
use crate::{Request, Response, StatusCode};

/// The routing table used by `Server`
///
/// Internally, we have a separate state machine per http method; indexing
/// by the method first allows the table itself to be more efficient.
#[allow(missing_debug_implementations)]
pub(crate) struct Router {
    method_map: HashMap<http_types::Method, MethodRouter<Arc<DynEndpoint>>>,
    all_method_router: MethodRouter<Arc<DynEndpoint>>,
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
    pub(crate) endpoint: Arc<DynEndpoint>,
    pub(crate) params: Captures<'static, 'static>,
}

impl Router {
    pub(crate) fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            all_method_router: MethodRouter::new(),
        }
    }

    pub(crate) fn add(&mut self, path: &str, method: http_types::Method, ep: Arc<DynEndpoint>) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, ep)
            .unwrap()
    }

    pub(crate) fn add_all(&mut self, path: &str, ep: Arc<DynEndpoint>) {
        self.all_method_router.add(path, ep).unwrap()
    }

    pub(crate) fn route(&self, path: &str, method: http_types::Method) -> Selection {
        if let Some(m) = self
            .method_map
            .get(&method)
            .and_then(|r| r.best_match(path))
        {
            Selection {
                endpoint: m.handler().to_owned(),
                params: m.captures().into_owned(),
            }
        } else if let Some(m) = self.all_method_router.best_match(path) {
            Selection {
                endpoint: m.handler().to_owned(),
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
                endpoint: Arc::new(method_not_allowed),
                params: Captures::default(),
            }
        } else {
            Selection {
                endpoint: Arc::new(not_found_endpoint),
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
