#![feature(async_await)]
use std::{
    cell::{ RefCell, Ref, RefMut },
    sync::Arc
};
use tide::{
    Context,
    Response,
    cookies::ContextExt,
    middleware::{ Middleware, Next }
};
use futures::future::BoxFuture;
use futures::prelude::*;
use http::header::HeaderValue;

mod session_map;
use self::session_map::SessionMap;

pub trait SessionStore {
    fn load_session(&self, key: &str) -> SessionMap;
    fn create_session(&self) -> SessionMap {
        SessionMap::new()
    }
    fn commit(&self, key: Option<&str>, session: Ref<Box<SessionMap>>) -> Result<HeaderValue, std::io::Error>;
}

pub struct SessionMiddleware<Store: SessionStore + Send + Sync> {
    pub session_key: String,
    pub store: Store
}

#[derive(Clone)]
pub struct SessionCell(RefCell<Box<SessionMap>>);

// We're copying actix, here. I need to understand this better, because
// this strikes me as dangerous.
#[doc(hidden)]
unsafe impl Send for SessionCell {}
#[doc(hidden)]
unsafe impl Sync for SessionCell {}

// If a handler needs access to the session (mutably or immutably) it can
// import this trait.
pub trait SessionExt {
    fn session(&self) -> Ref<Box<SessionMap>>;
    fn session_mut(&self) -> RefMut<Box<SessionMap>>;
}

impl<
    Data: Clone + Send + Sync + 'static
> SessionExt for Context<Data> {
    fn session(&self) -> Ref<Box<SessionMap>> {
        let session_cell = self.extensions().get::<Arc<SessionCell>>().unwrap();
        session_cell.0.borrow()
    }

    fn session_mut(&self) -> RefMut<Box<SessionMap>> {
        let session_cell = self.extensions().get::<Arc<SessionCell>>().unwrap();
        session_cell.0.borrow_mut()
    }
}

impl<
    Data: Clone + Send + Sync + 'static,
    S: SessionStore + Send + Sync + 'static
> Middleware<Data> for SessionMiddleware<S> {
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

            // Create a ref-counted cell (yay interior mutability.) Attach
            // a clone of that arc'd cell to the context and send it
            // through. At the same time, keep our local copy of the arc
            // ready for inspection after we're done processing the
            // request. 
            let cell = Arc::new(SessionCell(RefCell::new(Box::new(session))));
            ctx.extensions_mut().insert(cell.clone());
            let mut res = next.run(ctx).await;

            // Borrow the session map and check to see if we need to commit
            // it and/or send a new cookie.
            let session_cell = &cell.0;
            let session = session_cell.borrow();
            if !SessionMap::dirty(&session) {
                return res
            }

            if let Ok(key) = self.store.commit(session_key.as_ref().map(String::as_str), session) {
                if session_key.is_none() {
                    let hm = res.headers_mut();
                    // TODO: set the cookie's domain, path, expiry, sameSite,
                    // etc. properties via options set in the middleware.
                    // TODO: is there a good way to play nicely with cookie
                    // middleware? Can we rely on additions to context that
                    // other middleware add during the response half of the
                    // request lifecycle?
                    hm.insert("Set-Cookie", key);
                }
            }

            res
        })
    }
}

#[cfg(test)]
mod tests {
}
