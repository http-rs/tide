use tide::*;

struct CustomError;

impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        Response::new(418)
    }
}

#[test]
fn endpoint_with_custom_error() {
    fn run(/* req: Request<()> */) -> Result<String, CustomError> {
        Err(CustomError)
    }

    assert_eq!(run().into_response().status(), 418);
}

#[test]
fn endpoint_with_generic_error() {
    fn func() -> Result<(), String> {
        Err(String::from("error"))
    }

    fn run(/* req: Request<()> */) -> tide::Result<String> {
        func()?;

        Ok(String::from("ok"))
    }

    assert_eq!(run().into_response().status(), 500);

    fn run2(/* req: Request<()> */) -> tide::Result<String> {
        func().with_empty(400)?;

        Ok(String::from("ok"))
    }

    assert_eq!(run2().into_response().status(), 400);
}
