use crate::utils::BoxFuture;
use crate::{Endpoint, Request, Response};

/// Redirect a route permanently to another route.
///
/// The route will be permanently with a `301, permanent redirect` on a route
/// with the same HTTP method.
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
/// app.at("/nori").get(redirect::permanent("/"));
/// app.listen("127.0.0.1:8080").await?;
/// #
/// # Ok(()) }) }
/// ```
pub fn permanent(location: impl AsRef<str>) -> PermanentRedirect {
    let location = location.as_ref().to_owned();
    PermanentRedirect { location }
}

/// A permanent redirection endpoint.
#[derive(Debug, Clone)]
pub struct PermanentRedirect {
    location: String,
}

impl<State> Endpoint<State> for PermanentRedirect {
    fn call<'a>(&'a self, _req: Request<State>) -> BoxFuture<'a, crate::Result<Response>> {
        let res = Response::redirect_permanent(&self.location);
        Box::pin(async move { Ok(res) })
    }
}
