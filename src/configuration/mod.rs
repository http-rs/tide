//! Types for managing and extracting configuration.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use futures::future::FutureObj;

use crate::{Extract, Request, Response, RouteMatch};

mod default_config;

pub use self::default_config::{Configuration, ConfigurationBuilder};

trait StoreItem: Any + Send + Sync {
    fn clone_any(&self) -> Box<dyn StoreItem>;
    fn as_dyn_any(&self) -> &(dyn Any + Send + Sync);
    fn as_dyn_any_mut(&mut self) -> &mut (dyn Any + Send + Sync);
}

impl<T> StoreItem for T
where
    T: Any + Clone + Send + Sync,
{
    fn clone_any(&self) -> Box<dyn StoreItem> {
        Box::new(self.clone())
    }

    fn as_dyn_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn as_dyn_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }
}

impl Clone for Box<dyn StoreItem> {
    fn clone(&self) -> Box<dyn StoreItem> {
        (&**self).clone_any()
    }
}

/// A cloneable typemap for saving per-endpoint configuration.
///
/// Store is mostly managed by `App` and `Router`, so this is normally not used directly.
#[derive(Clone)]
pub struct Store(HashMap<TypeId, Box<dyn StoreItem>>);

impl Store {
    pub(crate) fn new() -> Self {
        Store(HashMap::new())
    }

    pub(crate) fn merge(&mut self, base: &Store) {
        let overlay = std::mem::replace(&mut self.0, base.0.clone());
        self.0.extend(overlay);
    }

    /// Retrieve the configuration item of given type, returning `None` if it is not found.
    pub fn read<T: Any + Clone + Send + Sync>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        self.0
            .get(&id)
            .and_then(|v| (**v).as_dyn_any().downcast_ref::<T>())
    }

    /// Save the given configuration item.
    pub fn write<T: Any + Clone + Send + Sync>(&mut self, value: T) {
        let id = TypeId::of::<T>();
        self.0.insert(id, Box::new(value) as Box<dyn StoreItem>);
    }
}

/// An extractor for reading configuration from endpoints.
///
/// It will try to retrieve the given configuration item. If it is not set, the extracted value
/// will be `None`.
pub struct ExtractConfiguration<T>(pub Option<T>);

impl<S: 'static, T: Any + Clone + Send + Sync + 'static> Extract<S> for ExtractConfiguration<T> {
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        config: &Store,
    ) -> Self::Fut {
        // The return type here is Option<K>, but the return type of the result of the future is
        // Result<ExtractConfiguration<T>, Response>, so rustc can infer that K == T, so we do not
        // need config.read::<T>().cloned()
        let config_item = config.read().cloned();
        FutureObj::new(Box::new(
            async move { Ok(ExtractConfiguration(config_item)) },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_read_write() {
        let mut config = Store::new();
        assert_eq!(config.read::<usize>(), None);
        assert_eq!(config.read::<isize>(), None);
        config.write(42usize);
        config.write(-3isize);
        assert_eq!(config.read::<usize>(), Some(&42));
        assert_eq!(config.read::<isize>(), Some(&-3));
        config.write(3usize);
        assert_eq!(config.read::<usize>(), Some(&3));
    }

    #[test]
    fn configuration_clone() {
        let mut config = Store::new();
        config.write(42usize);
        config.write(String::from("foo"));

        let mut new_config = config.clone();
        new_config.write(3usize);
        new_config.write(4u32);

        assert_eq!(config.read::<usize>(), Some(&42));
        assert_eq!(config.read::<u32>(), None);
        assert_eq!(config.read::<String>(), Some(&"foo".into()));

        assert_eq!(new_config.read::<usize>(), Some(&3));
        assert_eq!(new_config.read::<u32>(), Some(&4));
        assert_eq!(new_config.read::<String>(), Some(&"foo".into()));
    }
}
