use std::{
    cell::Ref,
    ops::{ Deref, DerefMut }
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct SessionMap {
    is_dirty: bool,
    data: HashMap<String, String> // XXX: this could be made more generic
}

// Provide associated functions a la Box or Arc, so we can
// Deref directly to the internal HashMap.
impl SessionMap {
    pub fn dirty(target: &Ref<Box<Self>>) -> bool {
        target.is_dirty
    }

    pub fn rotate(target: &mut Self) {
        target.is_dirty = true
    }

    pub fn new() -> Self {
        Self {
            is_dirty: false,
            data: HashMap::new()
        }
    }
}

impl Deref for SessionMap {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for SessionMap {
    // XXX: A tweet linked to this line in master earlier. If you're
    // coming in from that link, the original comment is preserved
    // at this URL: https://github.com/chrisdickinson/blog-rs/blob/6dfbe91a4fa09714ce6a975e4663e3e1efdaf9fa/src/session.rs#L45
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

