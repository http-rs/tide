use route_recognizer::{Match, Params, Router as MethodRouter};
use std::collections::HashMap;

use futures::future::BoxFuture;

use crate::endpoint::{DynEndpoint, Endpoint};
use crate::{Request, Response};

/// The routing table used by `Server`
///
/// Internally, we have a separate state machine per http method; indexing
/// by the method first allows the table itself to be more efficient.
#[allow(missing_debug_implementations)]
pub(crate) struct Router<State> {
    method_map: HashMap<http::Method, MethodRouter<Box<DynEndpoint<State>>>>,
    all_method_router: MethodRouter<Box<DynEndpoint<State>>>,
}

/// The result of routing a URL
pub(crate) struct Selection<'a, State> {
    pub(crate) endpoint: &'a DynEndpoint<State>,
    pub(crate) params: Params,
}

impl<State: 'static> Router<State> {
    pub(crate) fn new() -> Router<State> {
        Router {
            method_map: HashMap::default(),
            all_method_router: MethodRouter::new(),
        }
    }

    pub(crate) fn add(&mut self, path: &str, method: http::Method, ep: impl Endpoint<State>) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, Box::new(ep))
    }

    pub(crate) fn add_all(&mut self, path: &str, ep: impl Endpoint<State>) {
        self.all_method_router.add(path, Box::new(ep))
    }

    pub(crate) fn route(&self, path: &str, method: http::Method) -> Selection<'_, State> {
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
        } else if method == http::Method::HEAD {
            // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
            // if not then fallback to the behavior of HTTP GET else proceed as usual

            self.route(path, http::Method::GET)
        } else if self
            .method_map
            .iter()
            .filter(|(k, _)| *k != method)
            .any(|(_, r)| r.recognize(path).is_ok())
        {
            // If this `path` can be handled by a callback registered with a different HTTP method
            // should return 405 Method Not Allowed
            Selection {
                endpoint: &method_not_allowed,
                params: Params::new(),
            }
        } else {
            Selection {
                endpoint: &not_found_endpoint,
                params: Params::new(),
            }
        }
    }
}

fn not_found_endpoint<State>(_cx: Request<State>) -> BoxFuture<'static, Response> {
    Box::pin(async move { Response::new(http::StatusCode::NOT_FOUND.as_u16()) })
}

fn method_not_allowed<State>(_cx: Request<State>) -> BoxFuture<'static, Response> {
    Box::pin(async move { Response::new(http::StatusCode::METHOD_NOT_ALLOWED.as_u16()) })
}
