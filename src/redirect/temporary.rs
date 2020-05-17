use crate::utils::BoxFuture;
use crate::{Endpoint, Request, Response};

/// Redirect a route temporarily to another route.
///
/// The route will be redirected with a `307, temporary redirect` on a route with the same HTTP
/// method.
///
/// # Examples
/// ```no_run
/// # use async_std::task::block_on;
/// # fn main() -> Result<(), std::io::Error> { block_on(async {
/// #
/// use tide::redirect;
///
/// let mut app = tide::new();
/// app.at("/").get(|_| async move { Ok("meow") });
/// app.at("/nori").get(redirect::temporary("/"));
/// app.listen("127.0.0.1:8080").await?;
/// #
/// # Ok(()) }) }
/// ```
pub fn temporary(location: impl AsRef<str>) -> TemporaryRedirect {
    let location = location.as_ref().to_owned();
    TemporaryRedirect { location }
}

/// A temporary redirection endpoint.
#[derive(Debug, Clone)]
pub struct TemporaryRedirect {
    location: String,
}

impl<State> Endpoint<State> for TemporaryRedirect {
    fn call<'a>(&'a self, _req: Request<State>) -> BoxFuture<'a, crate::Result> {
        let res = Response::redirect_temporary(&self.location);
        Box::pin(async move { Ok(res) })
    }
}
