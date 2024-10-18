// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
//! files. For example, to override [LogSettings::tracing_level]:
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
use secrecy::{ExposeSecret, SecretString};
use serde_aux::field_attributes::deserialize_number_from_string;
use serde_derive::Deserialize;
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use std::env;
use std::time::Duration;
use tracing::level_filters::LevelFilter;

/// Name of the directory in which configuration files will be stored.
const CONF_DIR: &str = "config";

/// Top level `struct` for the configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    /// DB Settings.
    pub database: DataBaseSettings,
    /// email client settings.
    pub email_client: EmailClientSettings,
}

/// Application's settings.
#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationSettings {
    /// Listening port for the application.
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    /// Host address for the application.
    pub host: String,
    /// Base URL for accessing the application through the network.
    pub base_url: String,
    /// Log settings.
    pub log_settings: LogSettings,
}

/// Data Base connection settings.
#[derive(Clone, Debug, Deserialize)]
pub struct DataBaseSettings {
    /// Host address for the DB server.
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    /// Listening port for the DB server.
    pub port: u16,
    /// Username to access the application's database.
    pub username: String,
    /// Password to access the application's database.
    pub password: SecretString,
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

/// Log related settings.
///
/// # Description
///
/// This application outputs logs to a file by default. The file's name and path is
/// set via [LogSettings::log_output_file]. Output messages are append to the file if it exists
/// previously. Use **logrotate** or any other application to avoid ending with an
/// enormous log file.
///
/// Aside from that file, the application allows to output log messages to _stdout_
/// as well, useful for debugging sessions. This feature is enabled via
/// [LogSettings::enable_console_log].
///
/// Finally, the severity of the log messages to the console (when enabled) is also
/// configurable, thus it is allowed to set different severity levels for the regular
/// file log and the console log. This might be useful to avoid cluttering the console
/// output with too much information that could be read from the logfile.
#[derive(Clone, Debug, Deserialize)]
pub struct LogSettings {
    /// See [tracing::Level](https://docs.rs/tracing/0.1.40/tracing/struct.Level.html).
    /// Accepted values are specified at [LogSettings::get_verbosity_level].
    pub tracing_level: String,
    /// Output logs to a file. The value is the name of the output file.
    pub log_output_file: String,
    /// Enable console log output.
    pub enable_console_log: Option<bool>,
    /// Console verbosity.
    pub console_tracing_level: Option<String>,
}

/// Settings for the email client [mailjet_client](https://crates.io/crates/mailjet_client)
#[derive(Clone, Debug, Deserialize)]
pub struct EmailClientSettings {
    pub api_user: SecretString,
    pub api_key: SecretString,
    pub user_agent: String,
    pub target_api: String,
    pub admin_address: SecretString,
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
    pub fn connection_string(&self) -> SecretString {
        SecretString::from(format!(
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

    /// Build a connection to the MariaDB server without using a DB name.
    ///
    /// # Description
    ///
    /// The following settings will be applied:
    /// - [DataBaseSettings::host]
    /// - [DataBaseSettings::username]
    /// - [DataBaseSettings::password]
    /// - [DataBaseSettings::port]
    /// - [DataBaseSettings::require_ssl]
    pub fn build_db_conn_without_db(&self) -> MySqlConnectOptions {
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

    /// Build a connection to the MariaDB server without using a DB name.
    ///
    /// # Description
    ///
    /// The following settings will be applied plus the ones from [DataBaseSettings::build_db_conn_without_db]:
    /// - [DataBaseSettings::db_name]
    pub fn build_db_conn_with_db(&self) -> MySqlConnectOptions {
        self.build_db_conn_without_db().database(&self.db_name)
    }
}

impl LogSettings {
    /// Get the chosen verbosity level as a [LevelFilter] object.
    ///
    /// # Description
    ///
    /// Translate the tracing level that is given via a configuration file into a
    /// [LevelFilter] object. Such object can be passed straight to a `Subscriber` to
    /// specify a filter for the log messages.
    ///
    /// Accepted values:
    /// - `debug` or `dbg` to set the verbiosity to `DEBUG`.
    /// - `info` to set the verbosity to `INFO`.
    /// - `error` or `err` to set the verbosity to `ERROR`.
    /// - `trace` to set the verbosity to `TRACE`.
    /// - `warn` or any other string to set the verbosity to `WARN`.
    pub fn get_verbosity_level(&self) -> LevelFilter {
        LogSettings::verbosity(&self.tracing_level)
    }

    /// Get the chosen verbosity level for the console output as a [LevelFilter] object.
    ///
    /// # Description
    ///
    /// The tracing level for the console is not mandatory, thus it might be not present
    /// in the configuration file passed to the application. When no value was set,
    /// a severity level [LevelFilter::WARN] is returned.
    pub fn get_console_tracing_level(&self) -> LevelFilter {
        if let Some(level) = &self.console_tracing_level {
            LogSettings::verbosity(level)
        } else {
            LevelFilter::WARN
        }
    }

    /// Return if the console log was set via configuration file.
    pub fn console_log_enabled(&self) -> bool {
        self.enable_console_log.unwrap_or(false)
    }

    /// Translate a string into a [LevelFilter] or return a [LevelFilter::WARN] by default.
    fn verbosity(level: &str) -> LevelFilter {
        match level {
            "debug" | "dbg" => LevelFilter::DEBUG,
            "info" => LevelFilter::INFO,
            "error" | "err" => LevelFilter::ERROR,
            "trace" => LevelFilter::TRACE,
            _ => LevelFilter::WARN,
        }
    }
}
