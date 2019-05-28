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
    fn deref_mut(&mut self) -> &mut Self::Target {
        SessionMap::rotate(self);
        &mut self.data
    }
}
