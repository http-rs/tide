use futures::future::BoxFuture;
use futures::prelude::*;
use route_recognizer::{Match, Params, Router as MethodRouter};

use crate::http_service::Body;
use crate::middleware::{Request, Response, Context};
use super::Endpoint;

use std::collections::HashMap;

type DynEndpoint<State> =
    dyn (Fn(Request, State) -> BoxFuture<'static, Result<Response, std::io::Error>>) + 'static + Send + Sync;

/// The routing table used by `Server`
///
/// Internally, we have a separate state machine per http method; indexing
/// by the method first allows the table itself to be more efficient.
#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct Router<State> {
    method_map: HashMap<http::Method, MethodRouter<Box<DynEndpoint<State>>>>,
}

#[allow(missing_debug_implementations)]
pub struct Selection<'a, State> {
    endpoint: &'a DynEndpoint<State>,
    params: Params,
}

impl<State: 'static> Router<State> {
    pub(crate) fn new() -> Self {
        Self {
            method_map: HashMap::default(),
        }
    }

    pub(crate) fn add(&mut self, path: &str, method: http::Method, ep: impl Endpoint<State>) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, Box::new(move |cx| ep.call(cx).boxed()))
    }

    pub fn route(&self, path: &str, method: http::Method) -> Selection<'_, State> {
        let statement = self
            .method_map
            .get(&method)
            .and_then(|r| r.recognize(path).ok());

        if let Some(Match { handler, params }) = statement {
            Selection::new(&**handler, params)
        } else if method == http::Method::HEAD {
            // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
            // if not then fallback to the behavior of HTTP GET else proceed as usual

            self.route(path, http::Method::GET)
        } else {
            Selection::new(&not_found_endpoint, Params::new())
        }
    }
}

impl<'a, State> Selection<'a, State> {
    /// Create a new Selection
    pub(crate) fn new(endpoint: &'a DynEndpoint<State>, params: Params) -> Self {
        Self { endpoint, params }
    }

    /// Break Selection into it's components
    pub fn into_components(self) -> (&'a DynEndpoint<State>, Params) {
        (self.endpoint, self.params)
    }
}

fn not_found_endpoint<State>(_cx: Context<State>) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        http::Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    })
}

