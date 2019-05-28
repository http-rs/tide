#![feature(async_await)]
use std::cell::Ref;
use tide::{
    Context,
    Response,
    cookies::ContextExt,
    middleware::{ Middleware, Next }
};
use futures::future::BoxFuture;
use futures::prelude::*;
use http::header::HeaderValue;
use cookie::{ Cookie, CookieBuilder };

mod session_cell;
mod session_map;
mod ext;

pub use crate::session_map::SessionMap;
pub use crate::ext::SessionExt;

use self::session_cell::SessionCell;

pub trait SessionStore {
    fn load_session(&self, key: &str) -> SessionMap;
    fn create_session(&self) -> SessionMap {
        SessionMap::new()
    }
    fn commit(&self, key: Option<&str>, session: Ref<Box<SessionMap>>) -> Result<String, std::io::Error>;
}

pub struct SessionMiddleware<Store: SessionStore + Send + Sync, Configure: Send + Sync + Fn(CookieBuilder) -> CookieBuilder + 'static> {
    pub session_key: String,
    pub store: Store,

    pub configure_session_cookie: Configure
}

impl<
    Data: Clone + Send + Sync + 'static,
    S: SessionStore + Send + Sync + 'static,
    C: Send + Sync + Fn(CookieBuilder) -> CookieBuilder + 'static
> Middleware<Data> for SessionMiddleware<S, C> {
    fn handle<'a>(&'a self, mut ctx: Context<Data>, next: Next<'a, Data>) -> BoxFuture<'a, Response> {

        FutureExt::boxed(async move {
            let result_maybe_session = ctx.get_cookie(&self.session_key);

            let session_key = match result_maybe_session {
                Ok(maybe_session) => match maybe_session {
                    Some(cookie) => Some(String::from(cookie.value())),
                    None => None
                },
                Err(_) => None
            };

            let session = match session_key.as_ref() {
                Some(value) => {
                    self.store.load_session(&value)
                },
                None => self.store.create_session()
            };

            // Create a ref-counted cell. Attach a clone of that ARC'd cell to
            // the context and send it through. Meanwhile, keep our local copy
            // of the arc ready for inspection after we're done processing the
            // request. 
            let cell = SessionCell::new(session);
            ctx.extensions_mut().insert(cell.clone());
            let mut res = next.run(ctx).await;

            // Borrow the session map and check to see if we need to commit
            // it and/or send a new cookie.
            let session_cell = &cell.0;
            let session = session_cell.borrow();
            if !SessionMap::dirty(&session) {
                return res
            }

            if let Ok(sid) = self.store.commit(session_key.as_ref().map(String::as_str), session) {
                if session_key.is_none() {
                    let builder = Cookie::build(self.session_key.clone(), sid);
                    let c = (self.configure_session_cookie)(
                        builder
                    ).finish();

                    if let Ok(value) = HeaderValue::from_str(&c.to_string()) {
                        // TODO: is there a good way to play nicely with cookie
                        // middleware? Can we rely on additions to context that
                        // other middleware add during the response half of the
                        // request lifecycle?

                        let headers = res.headers_mut();
                        headers.insert("Set-Cookie", value);
                    }
                }
            }

            res
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{
        executor::{block_on, block_on_stream},
        stream::StreamExt,
    };
    use http_service::Body;
    use http_service_mock::make_server;
    use cookie::CookieBuilder;

    // Generates the app.
    fn app<S, C>(mw: SessionMiddleware<S, C>) -> tide::App<()> where
        S: SessionStore + Send + Sync + 'static,
        C: Fn(CookieBuilder) -> CookieBuilder + Send + Sync {
        let mut app = tide::App::new();
        app.at("/session/:key").get(async move |ctx: Context<()>| {
            let session = ctx.session();
            let key: String = ctx.param("key").expect("failed to parse url param");
            session.get(&key).expect("expected to be able to read key").clone()
        }).post(async move |mut ctx: Context<()>| {
            let key: String = ctx.param("key").expect("failed to parse url param");
            let body = ctx.body_string().await.expect("failed to read test request body");
            let mut session = ctx.session_mut();
            session.insert(key, body);
            "ok"
        });
        app.middleware(mw);
        app
    }

    fn configure_cookie (builder: CookieBuilder) -> CookieBuilder {
        builder
    }

    struct InMemorySessionStore;
    impl SessionStore for InMemorySessionStore {
        fn load_session(&self, key: &str) -> SessionMap {
            SessionMap::new()
        }

        fn commit(&self, key: Option<&str>, session: Ref<Box<SessionMap>>) -> Result<String, std::io::Error> {
            Ok(String::from("hi"))
        }
    }


    #[test]
    fn set_cookie_creates_new_session_id() {
        let app = app(SessionMiddleware {
            session_key: String::from("sid"),
            store: InMemorySessionStore { },
            configure_session_cookie: configure_cookie
        });
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::post("/session/testkey")
            .body(Body::from("hello world"))
            .unwrap();
        server.simulate(req).unwrap();
    }
}
