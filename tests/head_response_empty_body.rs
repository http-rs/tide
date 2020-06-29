use std::future::Future;
use std::pin::Pin;
use tide::http::{self, Method, Url};
use tide::{Body, Next, Request, Response, Result};

// this should be moved into tide, but not sure if as a middleware or hardcoded
fn head_body<'a, State: Send + Sync + 'static>(
    request: Request<State>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
    Box::pin(async {
        let method = request.method();
        let result = next.run(request).await;
        let mut response = result.unwrap_or_else(|e| Response::new(e.status()));

        if method == Method::Head {
            let body: &mut Body = response.as_mut();
            body.make_head();
        }

        Ok(response)
    })
}

#[async_std::test]
async fn head_response_empty() {
    let mut app = tide::new();
    app.at("/")
        .get(|_| async { Ok("this shouldn't exist in the body of a HEAD response") });

    app.middleware(head_body);
    let req = http::Request::new(Method::Head, Url::parse("http://example.com/").unwrap());
    let mut res: http::Response = app.respond(req).await.unwrap();

    let body = res.body_string().await.unwrap();
    assert!(body.is_empty());
}
