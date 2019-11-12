use async_std::task::{Context, Poll};
use async_std::future;

use std::pin::Pin;

use crate::{Request, Response, Endpoint};

/// Redirect a route to another route.
///
/// The route will be redirected with a `307, temporary redirect` on a route with the same HTTP
/// method.
///
/// # Examples
/// ```no_run
/// # use futures::executor::block_on;
/// # fn main() -> Result<(), std::io::Error> { block_on(async {
/// #
/// let mut app = tide::new();
/// app.at("/").get(|_| async move { "meow" });
/// app.at("/nori").get(tide::redirect("/"));
/// app.listen("127.0.0.1:8080").await?;
/// #
/// # Ok(()) }) }
/// ````
pub fn redirect<State>(location: impl AsRef<str>) -> impl Endpoint<State> {
    let location = location.as_ref().to_owned();
    Redirect { location }
}

/// The route that we redirect to
pub struct Redirect {
    location: String,
}

impl<State> Endpoint<State> for Redirect {
    type Fut = Future;

    fn call(&self, _req: Request<State>) -> Self::Fut {
        let res = Response::new(307).set_header("Location", &self.location);
        Future { res: Some(res) }
    }
}

/// Future returned from `redirect`.
pub struct Future {
    res: Option<Response>,
}

impl future::Future for Future {
    type Output = Response;
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.res.take().unwrap())
    }
}

