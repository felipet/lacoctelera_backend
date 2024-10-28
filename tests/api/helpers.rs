// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Common stuff for running integration tests.

use actix_web::rt::spawn;
use lacoctelera::{
    configuration::LogSettings,
    configuration::{DataBaseSettings, Settings},
    routes::ingredient::{FormData, QueryData},
    startup::Application,
    telemetry::configure_tracing,
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, MySqlConnection, MySqlPool};
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let mut settings = LogSettings {
        tracing_level: "info".into(),
        log_output_file: "debug".into(),
        enable_console_log: Some(true),
        console_tracing_level: Some("debug".to_string()),
    };

    if std::env::var("TEST_LOG").is_ok() {
        let level = std::env::var("TEST_LOG").expect("Failed to read the content of TEST_LOG var");
        match level.as_str() {
            "info" => settings.console_tracing_level = Some("info".into()),
            "debug" => settings.console_tracing_level = Some("debug".into()),
            "warn" => settings.console_tracing_level = Some("warn".into()),
            "error" => settings.console_tracing_level = Some("error".into()),
            &_ => settings.console_tracing_level = Some("none".into()),
        }

        if level != "none" {
            configure_tracing(&settings);
        }
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: MySqlPool,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub async fn get_ingredient(
        &self,
        query: &QueryData,
        parameter: Option<&str>,
    ) -> reqwest::Response {
        let param = parameter.unwrap_or("name");

        self.api_client
            .get(&format!(
                "{}/ingredient?{}={}",
                &self.address, param, query.name
            ))
            .send()
            .await
            .expect("Failed to execute get_ingredient request.")
    }

    pub async fn post_ingredient(&self, body: &FormData) -> reqwest::Response {
        self.api_client
            .post(&format!("{}/ingredient", &self.address))
            .json(body)
            .header("Content-type", "application/json")
            .send()
            .await
            .expect("Failed to execute post_ingredient request.")
    }

    pub async fn post_token_request<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/token/request", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute post_token_request.")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = Settings::new().expect("Failed to read configuration");
        c.database.db_name = Uuid::new_v4().to_string();
        // When using 0, a random port will be used.
        c.application.port = 0;
        c
    };

    // Connect to the DB backend.
    let db_pool = configure_database(&configuration.database).await;

    // Instantitate the backend application of La Coctelera.
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build La Coctelera application.");

    let port = application.port();
    let address = format!("{}:{port}", configuration.application.base_url);
    let _ = spawn(application.run_until_stopped());

    // Instantiate an HTTP client to run the tests against the app's backend.
    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    TestApp {
        address,
        db_pool,
        api_client,
    }
}

pub async fn configure_database(config: &DataBaseSettings) -> MySqlPool {
    // Connect to the testing DB without using a DB name, as we'll give a testing name.
    let mut conn = MySqlConnection::connect_with(&config.build_db_conn_without_db())
        .await
        .expect("Failed to connect to MariaDB.");

    conn.execute(format!(r#"CREATE DATABASE `{}`;"#, config.db_name).as_str())
        .await
        .expect("Failed to create test DB.");

    // Migrate the DB
    let conn_pool = MySqlPool::connect_with(config.build_db_conn_with_db())
        .await
        .expect("Failed to connect to MariaDB.");

    sqlx::migrate!("./migrations")
        .run(&conn_pool)
        .await
        .expect("Failed to migrate the testing DB.");

    conn_pool
}
