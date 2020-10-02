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

async fn login(req: Request<()>) -> tide::Result<impl Into<Response>> {
    let please_login = || {
        let schema = AuthenticationScheme::Basic;
        let realm = "access the tuna mainframe";
        let auth = WwwAuthenticate::new(schema, realm.into());
        let mut res = Response::new(401);
        auth.apply(&mut res);
        return Ok(res);
    };

    // Tell the client to authenticate
    if let None = req.header("Authorization") {
        return please_login();
    }
    let auth = match BasicAuth::from_headers(req)? {
        Some(auth) => auth,
        None => return please_login(),
    };

    dbg!(&auth);

    ensure_eq!(auth.username(), "nori", 401, "unknown username");
    ensure_eq!(auth.password(), "ilovefish", 401, "incorrect password");

    Ok("login successful!".into())
}
