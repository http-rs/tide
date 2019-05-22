//! Todo MVC backend example
//!
//! https://www.todobackend.com/specs/index.html

#![feature(async_await)]

mod cors;
mod routes {
    use super::cors;
    pub use tide::{Context, EndpointResult};

    pub fn setup(mut app: tide::App<()>) -> tide::App<()> {
        app.middleware(cors::CorsBlanket::new());
        app.at("/").get(get_todos);
        app.at("/").post(noop);
        app.at("/").delete(noop);
        app.at("/:todo").get(noop);
        app.at("/:todo").patch(noop);
        app.at("/:todo").delete(noop);
        app
    }

    async fn noop(_cx: tide::Context<()>) -> String {
        "{}".to_string()
    }

    pub async fn get_todos(_cx: Context<()>) -> String {
        String::from("hello world")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let app = tide::App::new();
    let app = routes::setup(app);
    app.run("localhost:8080")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use http_service::Response;
    use http_service_mock::make_server;
    use std::error::Error;
    use std::io;

    use super::routes;

    struct HttpTest<State>{
        req: http::request::Builder,
        app: tide::App<State>,
    }

    impl<State: Send + Sync + 'static> HttpTest<State> {
        pub fn new(app: tide::App<State>) -> Self {
            Self {
                app,
                req: http::Request::builder(),
            }
        }

        pub fn method(mut self, input: &str) -> Self{
            self.req.method(input);
            self
        }

        pub fn uri(mut self, input: &str) -> Self{
            self.req.uri(input);
            self
        }

        pub async fn send(mut self) -> Result<HttpTestResponse, Box<Error + Send + Sync + 'static>> {
            let req = self.req.body(http_service::Body::empty())?;
            let mut svc = make_server(self.app.into_http_service())?;
            let res = svc.simulate(req)?;
            Ok(HttpTestResponse { res })
        }
    }

    struct HttpTestResponse {
        res: Response
    }

    impl HttpTestResponse {
        fn status(self, status: u16) -> Self {
            assert_eq!(self.res.status(), status);
            self
        }

        fn header(self, key: &str, value: &'static str) -> Self {
            let value = http::header::HeaderValue::from_static(value);
            let header = self.res.headers().get(key);
            let header = header.expect("Header did not exist in the map");
            assert_eq!(header, value);
            self
        }

        async fn body(self) -> io::Result<Vec<u8>> {
            let body = self.res.into_body().into_vec().await?;
            Ok(body)
        }

        async fn body_string(self) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
            let body = self.body().await?;
            Ok(String::from_utf8(body)?)
        }
    }

    #[runtime::test]
    async fn cors() -> Result<(), Box<std::error::Error + Send + Sync + 'static>> {
        let app = routes::setup(tide::App::new());

        let response = HttpTest::new(app)
            .method("OPTIONS")
            .uri("http://localhost:8080")
            .send()
            .await?;

        let body = response
            .status(200)
            .header("access-control-allow-origin", "*")
            .header("access-control-allow-headers", "*")
            .body_string()
            .await?;

        assert_eq!(&body, "");
        Ok(())
    }

    #[runtime::test]
    async fn post_todo() -> Result<(), Box<std::error::Error + Send + Sync + 'static>> {
        let app = routes::setup(tide::App::new());

        let response = HttpTest::new(app)
            .method("POST")
            .uri("http://localhost:8080")
            .send()
            .await?;

        let body = response
            .status(200)
            .body_string()
            .await?;

        assert_eq!(&body, "{}");

        Ok(())
    }
}
