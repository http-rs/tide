use std::collections::BTreeMap;

use futures::future::FutureObj;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

use crate::{Extract, Request, Response, RouteMatch};

pub trait ConfigurationItem: Serialize + Deserialize<'static> {
    const NAME: &'static str;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Configuration(BTreeMap<String, toml::Value>);

impl Configuration {
    pub(crate) fn new() -> Self {
        Configuration(BTreeMap::new())
    }

    // TODO: properly handle errors
    pub fn read<T: ConfigurationItem>(&self) -> Option<T> {
        let value = self.0.get(T::NAME)?;
        value.clone().try_into::<T>().ok()
    }

    pub fn write<T: ConfigurationItem>(&mut self, value: T) -> Result<Option<T>, toml::ser::Error> {
        let value = toml::Value::try_from(value)?;
        let previous_value = self.0.insert(T::NAME.into(), value);
        Ok(previous_value.and_then(|v| v.try_into::<T>().ok()))
    }
}

pub struct ExtractConfiguration<T>(pub Option<T>);

impl<S: 'static, T: 'static + ConfigurationItem + Send> Extract<S> for ExtractConfiguration<T> {
    type Fut = FutureObj<'static, Result<Self, Response>>;

    fn extract(
        data: &mut S,
        req: &mut Request,
        params: &Option<RouteMatch<'_>>,
        config: &Configuration,
    ) -> Self::Fut {
        let config_item = config.read();
        FutureObj::new(Box::new(
            async move { Ok(ExtractConfiguration(config_item)) },
        ))
    }
}
