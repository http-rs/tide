// Originally from https://github.com/http-rs/http-types/blob/main/src/extensions.rs
//
// Implementation is based on
// - https://github.com/trillium-rs/trillium/blob/main/http/src/state_set.rs
// - https://github.com/hyperium/http/blob/master/src/extensions.rs
// - https://github.com/kardeiz/type-map/blob/master/src/lib.rs
use std::{
    any::{Any, TypeId},
    hash::{BuildHasherDefault, Hasher},
};

use hashbrown::HashMap;

/// Store and retrieve values by
/// [`TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html). This
/// allows storing arbitrary data that implements `Sync + Send +
/// 'static`.
#[derive(Default, Debug)]
pub struct State(HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>);

// With TypeIds as keys, there's no need to hash them. So we simply use an identy hasher.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

impl State {
    /// Create an empty `StateSet`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a `State` with a default inserted value.
    pub fn with<S: Send + Sync + 'static>(val: S) -> Self {
        let mut state = State::new();
        state.insert(val);
        state
    }

    /// Insert a value into this `State`.
    ///
    /// If a value of this type already exists, it will be returned.
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.0
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| (boxed as Box<dyn Any>).downcast().ok().map(|boxed| *boxed))
    }

    /// Get a reference to a value previously inserted on this `State`.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any)).downcast_ref())
    }

    /// Get a mutable reference to a value previously inserted on this `State`.
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any)).downcast_mut())
    }
}
