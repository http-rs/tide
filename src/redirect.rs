use crate::utils::BoxFuture;
use http_types::StatusCode;

use crate::{Endpoint, Request, Response};

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
/// ```
pub fn redirect<State>(location: impl AsRef<str>) -> impl Endpoint<State> {
    let location = location.as_ref().to_owned();
    Redirect { location }
}

/// The route that we redirect to
pub struct Redirect {
    location: String,
}

impl<State> Endpoint<State> for Redirect {
    fn call<'a>(&'a self, _req: Request<State>) -> BoxFuture<'a, crate::Result<Response>> {
        let res = Response::new(StatusCode::TemporaryRedirect)
            .set_header("location".parse().unwrap(), self.location.clone());
        Box::pin(async move { Ok(res) })
    }
}
