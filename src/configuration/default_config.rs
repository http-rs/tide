use structopt::StructOpt;

use std::fmt;

/// What environment are we running in?
#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Environment::Development => write!(f, "Development"),
            Environment::Staging => write!(f, "Staging"),
            Environment::Production => write!(f, "Production"),
        }
    }
}

fn parse_env(s: &str) -> Environment {
    match s {
        "development" => Environment::Development,
        "staging" => Environment::Staging,
        "production" => Environment::Production,
        // Default to development environment
        _ => Environment::Development,
    }
}

/// Default configuration for the application
///
/// Only the one that is applied to the top-level router will be regarded. Overriding this item in
/// resource paths or subrouters has no effect.
#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "Configuration")]
pub struct Configuration {
    /// Execution environment of the application
    #[structopt(
        short = "e",
        long = "env",
        default_value = "development",
        env = "ENV",
        parse(from_str = "parse_env")
    )]
    pub env: Environment,
    /// Address the server binds to
    #[structopt(short = "a", long = "addr", env = "ADDR", default_value = "127.0.0.1")]
    pub address: String,
    /// Port the server binds to
    #[structopt(short = "p", long = "port", env = "PORT", default_value = "8086")]
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

/// Holds application specific configuration
///
/// This struct implements the builder pattern, therefore you can override the default values
/// before calling `app.serve` by doing something like this:
///
/// ```rust, no_run
/// let mut app = tide::App::new(());
/// let updated_conf = tide::configuration::Configuration::build()
///     .port(8000)
///     .env(tide::configuration::Environment::Production)
///     .finalize();
/// app.config(updated_conf);
/// ```
///
/// Now the applivation will be running with the `Production` environment and will be listening on
/// port `8000` instead of the default `8086`
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
