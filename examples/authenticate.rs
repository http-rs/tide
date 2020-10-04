use tide::http::auth::{AuthenticationScheme, BasicAuth, WwwAuthenticate};
use tide::http::ensure_eq2 as ensure_eq;
use tide::sessions::{MemoryStore, SessionMiddleware};
use tide::{Request, Response};

const SECRET: &[u8] = b"shhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh";
const USERNAME: &str = "nori";
const PASSWORD: &str = "ilovefish";
const USER_ID: &str = "12";

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();
    app.with(SessionMiddleware::new(MemoryStore::new(), SECRET));
    app.at("/").get(login);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn login(mut req: Request<()>) -> tide::Result {
    let session = req.session();
    if let Some(_) = session.get::<String>("user_id") {
        return Ok("Already logged in!".into());
    }

    let auth = match BasicAuth::from_headers(&req)? {
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

    ensure_eq!(auth.username(), USERNAME, 401, "unknown username");
    ensure_eq!(auth.password(), PASSWORD, 401, "incorrect password");

    let session = req.session_mut();
    session.insert("user_id", USER_ID)?;

    Ok("Login successful!".into())
}
