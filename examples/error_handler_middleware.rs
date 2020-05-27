use std::convert::TryInto;
use tide::http::url::{self, Url};
use tide::{error::ErrorHandler, Request, Response, StatusCode};

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::with_level(tide::log::LevelFilter::Trace);
    let mut app = tide::new();

    app.middleware(ErrorHandler::new(
        //there could be any number of these, and they'd be executed in order specified
        |err: url::ParseError| async move {
            let mut response = Response::new(StatusCode::ImATeapot);
            response.set_body(err.to_string());
            Ok(response)
        },
    ));

    app.at("/").get(|_req: tide::Request<_>| async move {
        let _bad_parse = Url::parse("not a url")?; //intentionally errors
        Ok("this will return 418 I'm a teapot because we have a handler middleware")
    });

    app.at("/uncaught").get(|_req: Request<_>| async move {
        let _bad_parse: StatusCode = 800_u16.try_into()?;
        Ok("this will return a 500 because we don't have a handler for this error type")
    });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
