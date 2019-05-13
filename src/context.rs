use http::{HeaderMap, Method, Uri, Version};
use http_service::Body;
use route_recognizer::Params;
use std::{str::FromStr, sync::Arc};

/// Data associated with a request-response lifecycle.
///
/// The `Context` gives endpoints access to basic information about the incoming
/// request, route parameters, and various ways of accessing the request's body.
///
/// Contexts also provide *extensions*, a type map primarily used for low-level
/// communication between middleware and endpoints.
#[derive(Debug)]
pub struct Context<State> {
    state: Arc<State>,
    request: http_service::Request,
    route_params: Params,
}

impl<State> Context<State> {
    pub(crate) fn new(
        state: Arc<State>,
        request: http::Request<Body>,
        route_params: Params,
    ) -> Context<State> {
        Context {
            state,
            request,
            route_params,
        }
    }

    /// Access the request's HTTP method.
    pub fn method(&self) -> &Method {
        self.request.method()
    }

    /// Access the request's full URI method.
    pub fn uri(&self) -> &Uri {
        self.request.uri()
    }

    /// Access the request's HTTP version.
    pub fn version(&self) -> Version {
        self.request.version()
    }

    /// Access the request's headers.
    pub fn headers(&self) -> &HeaderMap {
        self.request.headers()
    }

    /// Access the entire request.
    pub fn request(&self) -> &http_service::Request {
        &self.request
    }

    ///  Access app-global data.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Extract and parse a route parameter by name.
    ///
    /// Returns the results of parsing the parameter according to the inferred
    /// output type `T`.
    ///
    /// The name should *not* include the leading `:` or the trailing `*` (if
    /// any).
    ///
    /// # Errors
    ///
    /// Yields an `Err` if the parameter was found but failed to parse as an
    /// instance of type `T`.
    ///
    /// # Panics
    ///
    /// Panic if `key` is not a parameter for the route.
    pub fn param<T: FromStr>(&self, key: &str) -> Result<T, T::Err> {
        self.route_params.find(key).unwrap().parse()
    }

    /// Reads the entire request body into a byte buffer.
    ///
    /// This method can be called after the body has already been read, but will
    /// produce an empty buffer.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    pub async fn body_bytes(&mut self) -> std::io::Result<Vec<u8>> {
        self.take_body().into_vec().await
    }

    /// Reads the entire request body into a string.
    ///
    /// This method can be called after the body has already been read, but will
    /// produce an empty buffer.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    ///
    /// If the body cannot be interpreted as valid UTF-8, an `Err` is returned.
    pub async fn body_string(&mut self) -> std::io::Result<String> {
        let body_bytes = self.body_bytes().await?;
        Ok(String::from_utf8(body_bytes).map_err(|_| std::io::ErrorKind::InvalidData)?)
    }

    /// Reads and deserialized the entire request body via json.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered while reading the body is immediately returned
    /// as an `Err`.
    ///
    /// If the body cannot be interpreted as valid json for the target type `T`,
    /// an `Err` is returned.
    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> std::io::Result<T> {
        let body_bytes = self.body_bytes().await?;
        Ok(serde_json::from_slice(&body_bytes).map_err(|_| std::io::ErrorKind::InvalidData)?)
    }

    /// Remove ownership of the request body, replacing it with an empty body.
    ///
    /// Used primarily for working directly with the body stream.
    pub fn take_body(&mut self) -> Body {
        std::mem::replace(self.request.body_mut(), Body::empty())
    }

    /// Access the extensions to the context.
    pub fn extensions(&self) -> &http::Extensions {
        self.request.extensions()
    }

    /// Mutably access the extensions to the context.
    pub fn extensions_mut(&mut self) -> &mut http::Extensions {
        self.request.extensions_mut()
    }
}
