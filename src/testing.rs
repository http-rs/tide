use crate::http::{Request, Response, Result, Url};
use crate::Server;
use futures_util::future::BoxFuture;
use std::sync::Arc;
use surf::{Client, HttpClient, RequestBuilder};

/// Trait that adds test request capabilities to tide [`Server`]s
pub trait TestingExt {
    /// Construct a new surf Client
    fn client(&self) -> Client;

    /// Builds a `CONNECT` request.
    fn connect(&self, path: &str) -> RequestBuilder {
        self.client().connect(path)
    }

    /// Builds a `DELETE` request.
    fn delete(&self, path: &str) -> RequestBuilder {
        self.client().delete(path)
    }

    /// Builds a `GET` request.
    fn get(&self, path: &str) -> RequestBuilder {
        self.client().get(path)
    }

    /// Builds a `HEAD` request.
    fn head(&self, path: &str) -> RequestBuilder {
        self.client().head(path)
    }

    /// Builds an `OPTIONS` request.
    fn options(&self, path: &str) -> RequestBuilder {
        self.client().options(path)
    }

    /// Builds a `PATCH` request.
    fn patch(&self, path: &str) -> RequestBuilder {
        self.client().patch(path)
    }

    /// Builds a `POST` request.
    fn post(&self, path: &str) -> RequestBuilder {
        self.client().post(path)
    }

    /// Builds a `PUT` request.
    fn put(&self, path: &str) -> RequestBuilder {
        self.client().put(path)
    }

    /// Builds a `TRACE` request.
    fn trace(&self, path: &str) -> RequestBuilder {
        self.client().trace(path)
    }
}

impl<State: Clone + Send + Sync + Unpin + 'static> TestingExt for Server<State> {
    fn client(&self) -> Client {
        let mut client = Client::with_http_client(Arc::new(self.clone()));
        client.set_base_url(Url::parse("http://example.com").unwrap());
        client
    }
}

impl<State: Clone + Send + Sync + Unpin + 'static> HttpClient for Server<State> {
    fn send(&self, req: Request) -> BoxFuture<'static, Result<Response>> {
        let self_cloned = self.clone();
        Box::pin(async move { self_cloned.respond(req).await })
    }
}
