use std::convert::{TryFrom, TryInto};
use sqlx::postgres::PgConnectOptions;

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize,Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize,Clone)]
pub struct EmailClientSettings{
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token:String
}

impl EmailClientSettings{
    pub fn sender(&self)-> Result<SubscriberEmail,String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

#[derive(serde::Deserialize,Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}
#[derive(serde::Deserialize,Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}
impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
    pub fn with_db(&self)-> PgConnectOptions{
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)

    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialize our configuration reader
    let mut settings = config::Config::default();

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    // Add configuration values from a file named `configuration`.
    // It will look for any top-level file with an extension
    // that `config` knows how to parse: yaml,json etc.

    // Read the "default" configuration file
    settings.merge(config::File::from(configuration_directory.join("base")).required(true))?;

    // Default the running environment
    // Default to `local` if unspecified
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    //Layer on the environment-specific values.
    settings.merge(
        config::File::from(configuration_directory.join(environment.as_str())).required(true),
    )?;

    // Add in settings from environment variables (with a prefix of APP and '__' as separator)
    // E.g. 'APP_APPLICATION__PORT=5001 would set 'Settings.application.port'
    settings.merge(config::Environment::with_prefix("app").separator("__"))?;
    // Try to convert the configuration values it read into
    // our Settings type
    settings.try_into()
}

/// the possible runtime environment for our application
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not supported environment. Use either 'local' or 'production'.",
                other
            )),
        }
    }
}
