//! Todo MVC backend example
//!
//! https://www.todobackend.com/specs/index.html

#![feature(async_await)]

mod cors;
mod routes {
    use super::cors;
    pub use tide::{Context, EndpointResult};

    pub fn setup(app: &mut tide::App<()>) {
        app.middleware(cors::CorsBlanket::new());
        app.at("/").get(get_todos);
        app.at("/").post(noop);
        app.at("/").delete(noop);
        app.at("/:todo").get(noop);
        app.at("/:todo").patch(noop);
        app.at("/:todo").delete(noop);
    }

    async fn noop(_cx: tide::Context<()>) -> String {
        "{}".to_string()
    }

    pub async fn get_todos(_cx: Context<()>) -> String {
        String::from("hello world")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut app = tide::App::new();
    routes::setup(&mut app);
    app.run("localhost:8080")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use futures::future::{BoxFuture, Ready};
    use http::request::Request;
    use http_service::{HttpService, Response};
    use http_service_mock::make_server;
    use std::error::Error;
    use std::io;

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

        pub fn send(mut self) -> Result<HttpTestResponse, Box<Error + Send + Sync + 'static>> {
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
        fn status(mut self, status: u16) -> Self {
            assert_eq!(self.res.status(), status);
            self
        }

        async fn body(mut self, body: &[u8]) -> io::Result<()> {
            assert_eq!(self.res.into_body().into_vec().await?, body);
            Ok(())
        }
    }

















    #[runtime::test]
    async fn index() -> Result<(), Box<std::error::Error + Send + Sync + 'static>> {
        let mut app = tide::App::new();
        super::routes::setup(&mut app);

        HttpTest::new(app)
            .method("GET")
            .uri("http://localhost:8080")
            .send()?
            .status(200)
            .body(b"hello world")
            .await?;

        Ok(())
    }
}
