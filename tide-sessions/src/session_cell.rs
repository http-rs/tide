use crate::session_map::SessionMap;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone)]
pub struct SessionCell(pub RefCell<Box<SessionMap>>);

// We're copying actix, here. I need to understand this better, because
// this strikes me as dangerous.
#[doc(hidden)]
unsafe impl Send for SessionCell {}
#[doc(hidden)]
unsafe impl Sync for SessionCell {}

impl SessionCell {
    pub fn new (map: SessionMap) -> Arc<Self> {
        Arc::new(Self(RefCell::new(Box::new(map))))
    }
}
