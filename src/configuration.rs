//! La Coctelera configuration module.
//!
//! # Description
//!
//! This module includes all the definitions for the app's settings and the
//! objects that automate reading the configuration from files or environment
//! variables and parsing them to Rust's native types.
//!
//! Some settings must be overridden by environment variables.
//! All the environment variables that are meant to be used within this module
//! shall use the prefix `LACOCTELERA`.
//!
//! # Settings
//!
//! The settings of the application may be set via 2 methods:
//! - Using the configuration files located in the `config` folder.
//! - Using environment variables.
//!
//! The former is advised for settings that usually take the same values and don't include
//! any value that shall not be exposed to the public (passwords, tokens, ...).
//! The latter is advised for settings that we only intend to set for a limited amount of
//! time, i.e. a debug session, or contain private values.
//!
//! ## Environment Variables
//!
//! The following environment variables are accepted by the application:
//!
//! - `RUN_MODE`: `devel`, `prod`. This variable shall take a value that refers to a
//!    configuration file in the `config` folder. The settings found there will
//!    overridden the settings found in `base.toml`. When not set, `prod` is considered
//!    as run mode.
//!
//! Variables defined within configuration files can be overridden using `LACOCTELERA`
//! prefix. Variables need to be scoped in the same way as they are found in the configuration
//! files. For example, to override [ApplicationSettings::tracing_level]:
//!
//! ```bash
//! $ LACOCTELERA__APPLICATION__TRACING_LEVEL=trace ./lacoctelera
//! ```
//!
//! **Note that the scope separator is a double `_`.**
//!
//! When multiple configuration variables are needed to be overridden, it is advised to
//! create a `local.toml` file within the `config` folder.
//!
//! ## Configuration Files
//!
//! All the required configuration variables are found in: `base.toml`, `prod.toml`
//! and `devel.toml`. The former refers to common settings that are usually applied
//! to both running scenarios: production and development. Any variable found there can
//! be overridden if defined on any of the latter files.
//!
//! The descriptions for each variable are found in the `Struct`s docs:
//! - [ApplicationSettings] for settings that apply to the main application.
//! - [DataBaseSettings] for settings that apply to the DB connection.

use config::{Config, ConfigError, Environment, File};
use core::time;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use serde_derive::Deserialize;
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use std::env;
use std::time::Duration;

/// Name of the directory in which configuration files will be stored.
const CONF_DIR: &str = "config";

/// Top level `struct` for the configuration.
#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
    /// DB Settings.
    pub database: DataBaseSettings,
}

/// Application's settings.
#[derive(Debug, Deserialize)]
pub struct ApplicationSettings {
    /// Listening port for the application.
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    /// Host address for the application.
    pub host: String,
    /// Base URL for accessing the application through the network.
    pub base_url: String,
    /// See [tracing::Level](https://docs.rs/tracing/0.1.40/tracing/struct.Level.html).
    pub tracing_level: String,
}

/// Data Base connection settings.
#[derive(Debug, Deserialize)]
pub struct DataBaseSettings {
    /// Host address for the DB server.
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    /// Listening port for the DB server.
    pub port: u16,
    /// Username to access the application's database.
    pub username: String,
    /// Password to access the application's database.
    pub password: Secret<String>,
    /// Name of the application's database.
    pub db_name: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    /// Maximum number of connections for the connections pool.
    pub max_connections: u16,
    /// Idle timeout for open connections.
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub idle_timeout_sec: u16,
    /// Force using SSL for the connection to the DB. False sets the connection to `Preferred` mode.
    pub require_ssl: bool,
}

impl Settings {
    /// Parse the application settings.
    pub fn new() -> Result<Self, ConfigError> {
        // Build the full path of the configuration directory.
        let base_path =
            std::env::current_dir().expect("Failed to determine the current directory.");
        let cfg_dir = base_path.join(CONF_DIR);

        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "devel".into());

        let settings = Config::builder()
            // Start of  by merging in the "default" configuration file.
            .add_source(File::from(cfg_dir.join("base")).required(true))
            .add_source(File::from(cfg_dir.join(run_mode)).required(false))
            .add_source(File::from(cfg_dir.join("local")).required(false))
            .add_source(Environment::with_prefix("lacoctelera").separator("__"))
            .build()?;

        settings.try_deserialize()
    }
}

impl DataBaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.db_name,
        ))
    }

    /// Translate a timeout in seconds from an integer to a type `time::Duration`.
    pub fn idle_timeout(&self) -> time::Duration {
        Duration::from_secs(self.idle_timeout_sec as u64)
    }

    /// Build a `MySqlConnection` using the given settings.
    pub fn build_db_conn(&self) -> MySqlConnectOptions {
        MySqlConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(if self.require_ssl {
                MySqlSslMode::Required
            } else {
                MySqlSslMode::Preferred
            })
    }
}
