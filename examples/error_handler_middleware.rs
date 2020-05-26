use tide::{Request, Response, Result, StatusCode};
#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::with_level(tide::log::LevelFilter::Trace);
    let mut app = tide::new();

    app.middleware(ErrorHandler::new(
        //there could be any number of these, and they'd be executed in order specified
        |_req: Request<_>, _err: tide::http::url::ParseError| {
            Ok(Response::new(StatusCode::ImATeapot))
        },
    ));

    app.at("/").get(|_req: tide::Request<_>| async move {
        let _bad_parse: tide::http::Url = "".parse()?; //intentionally errors
        Ok("this will return 418 I'm a teapot because we have a handler middleware")
    });

    app.at("/uncaught").get(|_req: Request<_>| async move {
        let _bad_parse: StatusCode = 800_u16.try_into()?;
        Ok("this will return a 500 because we don't have a handler for this error type")
    });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

//
//  everything below here would be moved into tide or an external crate if this approach were adopted
//

use std::convert::TryInto;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::pin::Pin;
use tide::{Middleware, Next};

struct ErrorHandler<E, S> {
    f: Box<dyn Fn(Request<S>, E) -> Result<Response> + Send + Sync + 'static>,
}

impl<E, S> Debug for ErrorHandler<E, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "ErrorHandler for {}",
            std::any::type_name::<E>()
        ))
    }
}

impl<E, S> ErrorHandler<E, S>
where
    E: Display + Debug + Send + Sync + 'static,
{
    fn new<F>(f: F) -> Self
    where
        F: Fn(Request<S>, E) -> Result<Response> + Send + Sync + 'static,
    {
        Self { f: Box::new(f) }
    }
}

impl<State: Clone + Send + Sync + 'static, E> Middleware<State> for ErrorHandler<E, State>
where
    E: Display + Debug + Send + Sync + 'static,
{
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
        Box::pin(async move {
            let req_cloned_this_is_problematic = req.clone();
            let response = next.run(req).await;
            if let Err(e) = response {
                e.downcast::<E>()
                    .and_then(move |e| (self.f)(req_cloned_this_is_problematic, e))
            } else {
                response
            }
        })
    }
}
