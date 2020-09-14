use std::sync::Arc;

use portpicker::pick_unused_port;
use surf::{Client, RequestBuilder};
use tide::http::url::Url;
use tide::Server;

/// Find an unused port.
#[allow(dead_code)]
pub async fn find_port() -> u16 {
    pick_unused_port().expect("No ports free")
}

#[async_trait::async_trait]
pub trait ServerTestingExt {
    fn client(&self) -> Client;
    fn connect(&self, path: &str) -> RequestBuilder;
    fn delete(&self, path: &str) -> RequestBuilder;
    fn get(&self, path: &str) -> RequestBuilder;
    fn head(&self, path: &str) -> RequestBuilder;
    fn options(&self, path: &str) -> RequestBuilder;
    fn patch(&self, path: &str) -> RequestBuilder;
    fn post(&self, path: &str) -> RequestBuilder;
    fn put(&self, path: &str) -> RequestBuilder;
    fn trace(&self, path: &str) -> RequestBuilder;
}

#[async_trait::async_trait]
impl<State> ServerTestingExt for Server<State>
where
    State: Unpin + Clone + Send + Sync + 'static,
{
    fn client(&self) -> Client {
        let mut client = Client::with_http_client(Arc::new(self.clone()));
        client.set_base_url(Url::parse("http://example.com").unwrap());
        client
    }

    fn connect(&self, path: &str) -> RequestBuilder {
        self.client().connect(path)
    }
    fn delete(&self, path: &str) -> RequestBuilder {
        self.client().delete(path)
    }
    fn get(&self, path: &str) -> RequestBuilder {
        self.client().get(path)
    }
    fn head(&self, path: &str) -> RequestBuilder {
        self.client().head(path)
    }
    fn options(&self, path: &str) -> RequestBuilder {
        self.client().options(path)
    }
    fn patch(&self, path: &str) -> RequestBuilder {
        self.client().patch(path)
    }
    fn post(&self, path: &str) -> RequestBuilder {
        self.client().post(path)
    }
    fn put(&self, path: &str) -> RequestBuilder {
        self.client().put(path)
    }
    fn trace(&self, path: &str) -> RequestBuilder {
        self.client().trace(path)
    }
}
