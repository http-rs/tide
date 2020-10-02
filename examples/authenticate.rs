use tide::http::auth::{AuthenticationScheme, BasicAuth, WwwAuthenticate};
use tide::http::ensure_eq2 as ensure_eq;
use tide::{Request, Response};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();
    app.at("/").get(login);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn login(req: Request<()>) -> tide::Result {
    let auth = match BasicAuth::from_headers(req)? {
        Some(auth) => auth,
        None => {
            let schema = AuthenticationScheme::Basic;
            let realm = "access the tuna mainframe";
            let auth = WwwAuthenticate::new(schema, realm.into());
            return Ok(Response::builder(401)
                .header(auth.name(), auth.value())
                .build());
        }
    };

    ensure_eq!(auth.username(), "nori", 401, "unknown username");
    ensure_eq!(auth.password(), "ilovefish", 401, "incorrect password");

    Ok("login successful!".into())
}
