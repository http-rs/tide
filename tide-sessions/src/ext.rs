use crate::session_cell::SessionCell;
use crate::session_map::SessionMap;
use std::cell::{ Ref, RefMut };
use std::sync::Arc;
use tide::Context;

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

