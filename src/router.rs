use route_recognizer::{Match, Params, Router as MethodRouter};
use std::collections::HashMap;

use crate::endpoint::DynEndpoint;
use crate::{Request, Response, StatusCode};

/// The routing table used by `Server`
///
/// Internally, we have a separate state machine per http method; indexing
/// by the method first allows the table itself to be more efficient.
#[allow(missing_debug_implementations)]
pub struct Router<State> {
    method_map: HashMap<http_types::Method, MethodRouter<Box<DynEndpoint<State>>>>,
    all_method_router: MethodRouter<Box<DynEndpoint<State>>>,
}

/// The result of routing a URL
#[allow(missing_debug_implementations)]
pub struct Selection<'a, State> {
    pub(crate) endpoint: &'a DynEndpoint<State>,
    pub(crate) params: Params,
}

impl<'a, State> Selection<'a, State>
where
    State: Clone + Send + Sync + 'static,
{
    pub fn not_found_endpoint() -> Selection<'a, State> {
        Selection {
            endpoint: &not_found_endpoint,
            params: Params::new(),
        }
    }

    pub fn method_not_allowed() -> Selection<'a, State> {
        Selection {
            endpoint: &method_not_allowed,
            params: Params::new(),
        }
    }
}

impl<State: Clone + Send + Sync + 'static> Router<State> {
    pub fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            all_method_router: MethodRouter::new(),
        }
    }

    pub fn add(&mut self, path: &str, method: http_types::Method, ep: Box<DynEndpoint<State>>) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, ep)
    }

    pub fn add_all(&mut self, path: &str, ep: Box<DynEndpoint<State>>) {
        self.all_method_router.add(path, ep)
    }

    pub fn route(&self, path: &str, method: http_types::Method) -> Selection<'_, State> {
        if let Some(Match { handler, params }) = self
            .method_map
            .get(&method)
            .and_then(|r| r.recognize(path).ok())
        {
            Selection {
                endpoint: &**handler,
                params,
            }
        } else if let Ok(Match { handler, params }) = self.all_method_router.recognize(path) {
            Selection {
                endpoint: &**handler,
                params,
            }
        } else if method == http_types::Method::Head {
            // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
            // if not then fallback to the behavior of HTTP GET else proceed as usual

            self.route(path, http_types::Method::Get)
        } else if self
            .method_map
            .iter()
            .filter(|(k, _)| **k != method)
            .any(|(_, r)| r.recognize(path).is_ok())
        {
            // If this `path` can be handled by a callback registered with a different HTTP method
            // should return 405 Method Not Allowed
            Selection::method_not_allowed()
        } else {
            Selection::not_found_endpoint()
        }
    }
}

async fn not_found_endpoint<State: Clone + Send + Sync + 'static>(
    _req: Request<State>,
) -> crate::Result {
    Ok(Response::new(StatusCode::NotFound))
}

async fn method_not_allowed<State: Clone + Send + Sync + 'static>(
    _req: Request<State>,
) -> crate::Result {
    Ok(Response::new(StatusCode::MethodNotAllowed))
}
