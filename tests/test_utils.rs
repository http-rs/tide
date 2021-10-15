use portpicker::pick_unused_port;

/// Find an unused port.
#[allow(dead_code)]
pub async fn find_port() -> u16 {
    pick_unused_port().expect("No ports free")
}

use surf::{Client, RequestBuilder};

/// Trait that adds test request capabilities to tide [`Server`]s
pub trait ServerTestingExt {
    /// Construct a new surf Client
    fn client(&self) -> Client;

    /// Builds a `CONNECT` request.
    fn connect(&self, uri: &str) -> RequestBuilder {
        self.client().connect(uri)
    }

    /// Builds a `DELETE` request.
    fn delete(&self, uri: &str) -> RequestBuilder {
        self.client().delete(uri)
    }

    /// Builds a `GET` request.
    fn get(&self, uri: &str) -> RequestBuilder {
        self.client().get(uri)
    }

    /// Builds a `HEAD` request.
    fn head(&self, uri: &str) -> RequestBuilder {
        self.client().head(uri)
    }

    /// Builds an `OPTIONS` request.
    fn options(&self, uri: &str) -> RequestBuilder {
        self.client().options(uri)
    }

    /// Builds a `PATCH` request.
    fn patch(&self, uri: &str) -> RequestBuilder {
        self.client().patch(uri)
    }

    /// Builds a `POST` request.
    fn post(&self, uri: &str) -> RequestBuilder {
        self.client().post(uri)
    }

    /// Builds a `PUT` request.
    fn put(&self, uri: &str) -> RequestBuilder {
        self.client().put(uri)
    }

    /// Builds a `TRACE` request.
    fn trace(&self, uri: &str) -> RequestBuilder {
        self.client().trace(uri)
    }
}

impl<ServerState: Clone + Send + Sync + Unpin + 'static> ServerTestingExt
    for tide::Server<ServerState>
{
    fn client(&self) -> Client {
        let mut client = Client::with_http_client(self.clone());
        client.set_base_url(tide::http::Url::parse("http://example.com").unwrap());
        client
    }
}
