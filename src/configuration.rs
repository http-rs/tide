use std::any::{Any, TypeId};
use std::collections::HashMap;

use futures::future::FutureObj;

use crate::{Extract, Request, Response, RouteMatch};

trait ConfigurationItem: Any + Send + Sync {
    fn clone_any(&self) -> Box<dyn ConfigurationItem>;
    fn as_dyn_any(&self) -> &(dyn Any + Send + Sync);
    fn as_dyn_any_mut(&mut self) -> &mut (dyn Any + Send + Sync);
}

impl<T> ConfigurationItem for T
where
    T: Any + Clone + Send + Sync,
{
    fn clone_any(&self) -> Box<dyn ConfigurationItem> {
        Box::new(self.clone())
    }

    fn as_dyn_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn as_dyn_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }
}

impl Clone for Box<dyn ConfigurationItem> {
    fn clone(&self) -> Box<dyn ConfigurationItem> {
        (&**self).clone_any()
    }
}

#[derive(Clone)]
pub struct Configuration(HashMap<TypeId, Box<dyn ConfigurationItem>>);

impl Configuration {
    pub(crate) fn new() -> Self {
        Configuration(HashMap::new())
    }

    pub fn read<T: Any + Clone + Send + Sync>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        self.0.get(&id).and_then(|v| {
            (**v).as_dyn_any().downcast_ref::<T>()
        })
    }

    pub fn write<T: Any + Clone + Send + Sync>(&mut self, value: T) {
        let id = TypeId::of::<T>();
        self.0.insert(id, Box::new(value) as Box<dyn ConfigurationItem>);
    }
}

pub struct ExtractConfiguration<T>(pub Option<T>);

impl<S: 'static, T: Any + Clone + Send + Sync + 'static> Extract<S> for ExtractConfiguration<T> {
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        config: &Configuration,
    ) -> Self::Fut {
        let config_item = config.read().cloned();
        FutureObj::new(Box::new(
            async move { Ok(ExtractConfiguration(config_item)) },
        ))
    }
}
