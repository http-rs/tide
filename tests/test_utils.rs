use portpicker::pick_unused_port;

use std::future::Future;
use std::pin::Pin;
use tide::http::{self, url::Url, Method};
use tide::Server;

/// An owned dynamically typed [`Future`] for use in cases where you can't
/// statically type your result or need to add some indirection.
#[allow(dead_code)]
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Find an unused port.
#[allow(dead_code)]
pub async fn find_port() -> u16 {
    pick_unused_port().expect("No ports free")
}

pub trait ServerTestingExt {
    fn request<'a>(
        &'a self,
        method: Method,
        path: &str,
    ) -> Pin<Box<dyn Future<Output = http::Response> + 'a + Send>>;
    fn request_body<'a>(
        &'a self,
        method: Method,
        path: &str,
    ) -> Pin<Box<dyn Future<Output = String> + 'a + Send>>;
    fn get<'a>(&'a self, path: &str) -> Pin<Box<dyn Future<Output = http::Response> + 'a + Send>>;
    fn get_body<'a>(&'a self, path: &str) -> Pin<Box<dyn Future<Output = String> + 'a + Send>>;
    fn post<'a>(&'a self, path: &str) -> Pin<Box<dyn Future<Output = http::Response> + 'a + Send>>;
    fn put<'a>(&'a self, path: &str) -> Pin<Box<dyn Future<Output = http::Response> + 'a + Send>>;
}

impl<State> ServerTestingExt for Server<State>
where
    State: Send + Sync + 'static,
{
    fn request<'a>(
        &'a self,
        method: Method,
        path: &str,
    ) -> Pin<Box<dyn Future<Output = http::Response> + 'a + Send>> {
        let url = if path.starts_with("http:") {
            Url::parse(path).unwrap()
        } else {
            Url::parse("http://example.com/")
                .unwrap()
                .join(path)
                .unwrap()
        };

        let request = http::Request::new(method, url);
        let fut = self.respond(request);
        Box::pin(async { fut.await.unwrap() })
    }

    fn request_body<'a>(
        &'a self,
        method: Method,
        path: &str,
    ) -> Pin<Box<dyn Future<Output = String> + 'a + Send>> {
        let response_fut = self.request(method, path);
        Box::pin(async move {
            let mut response = response_fut.await;
            response.body_string().await.unwrap()
        })
    }

    fn get<'a>(&'a self, path: &str) -> Pin<Box<dyn Future<Output = http::Response> + 'a + Send>> {
        self.request(Method::Get, path)
    }

    fn get_body<'a>(&'a self, path: &str) -> Pin<Box<dyn Future<Output = String> + 'a + Send>> {
        self.request_body(Method::Get, path)
    }

    fn post<'a>(&'a self, path: &str) -> Pin<Box<dyn Future<Output = http::Response> + 'a + Send>> {
        self.request(Method::Post, path)
    }

    fn put<'a>(&'a self, path: &str) -> Pin<Box<dyn Future<Output = http::Response> + 'a + Send>> {
        self.request(Method::Put, path)
    }
}
