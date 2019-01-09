/// What environment are we running in?
#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

/// Holds the default configuration for the App.
///
/// Only the one that is applied to the top-level router will be regarded. Overriding this item in
/// resource paths or subrouters has no effect.
#[derive(Debug, Clone)]
pub struct Configuration {
    pub env: Environment,
    pub address: String,
    pub port: u16,
}

pub struct ConfigurationBuilder {
    pub env: Environment,
    pub address: String,
    pub port: u16,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            env: Environment::Development,
            address: "127.0.0.1".to_owned(),
            port: 8181,
        }
    }
}

impl Configuration {
    pub fn build() -> ConfigurationBuilder {
        ConfigurationBuilder::default()
    }
}

impl Default for ConfigurationBuilder {
    fn default() -> Self {
        let config = Configuration::default();

        Self {
            env: config.env,
            address: config.address,
            port: config.port,
        }
    }
}

impl ConfigurationBuilder {
    pub fn env(mut self, env: Environment) -> Self {
        self.env = env;
        self
    }

    pub fn address<A: Into<String>>(mut self, address: A) -> Self {
        self.address = address.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn finalize(self) -> Configuration {
        let mut config = Configuration::default();

        config.port = self.port;
        config.address = self.address;
        config.env = self.env;

        config
    }
}
