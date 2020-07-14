use portpicker::pick_unused_port;
use tide::http::{self, url::Url, Method};
use tide::Server;

/// Find an unused port.
#[allow(dead_code)]
pub async fn find_port() -> u16 {
    pick_unused_port().expect("No ports free")
}

#[async_trait::async_trait]
pub trait ServerTestingExt {
    async fn request(&self, method: Method, path: &str) -> http::Response;
    async fn request_body(&self, method: Method, path: &str) -> String;
    async fn get(&self, path: &str) -> http::Response;
    async fn get_body(&self, path: &str) -> String;
    async fn post(&self, path: &str) -> http::Response;
    async fn put(&self, path: &str) -> http::Response;
}

#[async_trait::async_trait]
impl<State> ServerTestingExt for Server<State>
where
    State: Clone + Send + Sync + 'static,
{
    async fn request(&self, method: Method, path: &str) -> http::Response {
        let url = if path.starts_with("http:") || path.starts_with("https:") {
            Url::parse(path).unwrap()
        } else {
            Url::parse("http://example.com/")
                .unwrap()
                .join(path)
                .unwrap()
        };

        let request = http::Request::new(method, url);
        self.respond(request).await.unwrap()
    }

    async fn request_body(&self, method: Method, path: &str) -> String {
        let mut response = self.request(method, path).await;
        response.body_string().await.unwrap()
    }

    async fn get(&self, path: &str) -> http::Response {
        self.request(Method::Get, path).await
    }

    async fn get_body(&self, path: &str) -> String {
        self.request_body(Method::Get, path).await
    }

    async fn post(&self, path: &str) -> http::Response {
        self.request(Method::Post, path).await
    }

    async fn put(&self, path: &str) -> http::Response {
        self.request(Method::Put, path).await
    }
}
