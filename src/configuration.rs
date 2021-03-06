use crate::domain::SubscriberEmail;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::{
  postgres::{PgConnectOptions, PgSslMode},
  ConnectOptions,
};

#[derive(Clone, serde::Deserialize)]
pub struct ApplicationSettings {
  #[serde(deserialize_with = "deserialize_number_from_string")]
  pub port: u16,
  pub host: String,
  pub base_url: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct DatabaseSettings {
  pub username: String,
  pub password: Secret<String>,
  #[serde(deserialize_with = "deserialize_number_from_string")]
  pub port: u16,
  pub host: String,
  pub database_name: String,
  pub require_ssl: bool,
}

impl DatabaseSettings {
  pub fn without_db(&self) -> PgConnectOptions {
    let ssl_mode = if self.require_ssl {
      PgSslMode::Require
    } else {
      PgSslMode::Prefer
    };

    PgConnectOptions::new()
      .username(&self.username)
      .password(self.password.expose_secret())
      .host(&self.host)
      .port(self.port)
      .ssl_mode(ssl_mode)
  }

  pub fn with_db(&self) -> PgConnectOptions {
    let mut options = self.without_db().database(&self.database_name);
    options.log_statements(tracing::log::LevelFilter::Trace);
    options
  }
}

#[derive(Clone, serde::Deserialize)]
pub struct EmailClientSettings {
  pub base_url: String,
  pub sender_email: String,
  pub mailgun_api_key: Secret<String>,
  pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
  pub fn sender(&self) -> Result<SubscriberEmail, String> {
    SubscriberEmail::parse(self.sender_email.clone())
  }
  pub fn timeout(&self) -> std::time::Duration {
    std::time::Duration::from_millis(self.timeout_milliseconds)
  }
}

#[derive(Clone, serde::Deserialize)]
pub struct Settings {
  pub database: DatabaseSettings,
  pub application: ApplicationSettings,
  pub email_client: EmailClientSettings,
}

pub enum Environment {
  Local,
  Production,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
  let mut settings = config::Config::default();
  let base_path = std::env::current_dir().expect("Failed to determine the current directory");
  let config_directory = base_path.join("config");

  settings.merge(config::File::from(config_directory.join("base")).required(true))?;

  let env: Environment = std::env::var("APP_ENVIRONMENT")
    .unwrap_or_else(|_| "local".into())
    .try_into()
    .expect("Failed to parse APP_ENVIRONMENT");

  settings.merge(config::File::from(config_directory.join(env.as_str())).required(true))?;
  settings.merge(config::Environment::with_prefix("app").separator("__"))?;

  settings.try_into()
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
        "{} is not a supported environment. Use either `local` or `production`.",
        other
      )),
    }
  }
}
