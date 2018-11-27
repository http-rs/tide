use crate::app::App;
use tokio;

use std::sync::Arc;

use futures::{
    future::FutureObj,
    prelude::*,
};

use crate::{
    body::Body,
    router::{Router, RouteResult},
    endpoint::BoxedEndpoint,
    middleware::{RequestContext},
};

impl<Data: Clone + Send + Sync + 'static> App<Data> {
    pub fn into_test(self) -> Test<Data> {
        Test {
            data: self.data,
            router: Arc::new(self.router),
            default_handler: Arc::new(self.default_handler),
        }
    }
}

#[derive(Clone)]
pub struct Test<Data> {
    data: Data,
    router: Arc<Router<Data>>,
    default_handler: Arc<BoxedEndpoint<Data>>,
}

impl<Data: Clone + Send + Sync + 'static> Test<Data> {
    pub fn call(&mut self, req: http::Request<Body>) -> FutureObj<'static, Result<http::Response<Body>, std::io::Error>> {
        let data = self.data.clone();
        let router = self.router.clone();
        let default_handler = self.default_handler.clone();

        let req = req.map(Body::from);
        let path = req.uri().path().to_owned();
        let method = req.method().to_owned();

        FutureObj::new(Box::new(
            async move {
                let RouteResult {
                    endpoint,
                    params,
                    middleware,
                } = router.route(&path, &method, &default_handler);

                let ctx = RequestContext {
                    app_data: data,
                    req,
                    params,
                    endpoint,
                    next_middleware: middleware,
                };
                let res = await!(ctx.next());

                Ok(res.map(Into::into))
            },
        ))
    }

}

pub fn test<T, F>(test: T) -> impl Future<Output=Result<(), Box<std::any::Any + 'static + Send>>>
        where T: FnOnce() -> F,
              F: Future<Output=()> + Send + 'static
{
    FutureObj::new(Box::new(
        std::panic::AssertUnwindSafe(
            test()
        ).catch_unwind()
    ))
}

pub mod tokio_test {
    use futures::prelude::*;

    pub fn test<T, F>(test: T)
        where T: FnOnce() -> F,
              T: 'static,
              F: Future<Output=()> + Send + 'static
    {
        let runner = super::test(test).compat();

        tokio::runtime::Runtime::new().unwrap().block_on(runner).unwrap();
    }
}

#[macro_export]
macro_rules! assert_status {
    ($response:ident, $status:expr) => ({
        let status = $response.status();
        if !($response.status() == $status) {
            panic!(r#"assertion failed:
expected response code: `{:?}`,
received response code: `{:?}`"#, $status, status)
        }
    });
}

#[macro_export]
macro_rules! assert_ok {
    ($response:ident) => ({
        assert_status!($response, 200)
    });
}


#[cfg(test)]
mod test {
    use crate::app::App;
    use super::tokio_test::test;

    #[test]
    fn working_test() {
        let mut app = App::new(());
        app.at("/").get(async || "Hello, world!");
        let mut tester = app.into_test();

        let request = http::Request::builder()
            .method("GET")
            .body(Vec::new().into())
            .unwrap();

        test(
            async move || {
                let response = await!(tester.call(request)).unwrap();
                assert_ok!(response);
            }
        )
    }

    #[test]
    #[should_panic]
    fn failing_test() {
        let mut app = App::new(());
        app.at("/").get(async || "Hello, world!");
        let mut tester = app.into_test();

        let request = http::Request::builder()
            .method("POST")
            .body(Vec::new().into ())
            .unwrap();

        test(
            async move || {
                let response = await!(tester.call(request)).unwrap();
                assert_ok!(response);
            }
        )
    }
}