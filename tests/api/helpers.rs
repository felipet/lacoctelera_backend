// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Common stuff for running integration tests.

use actix_web::rt::spawn;
use lacoctelera::{
    authentication::{generate_new_token_hash, generate_token, store_validation_token, AuthData},
    configuration::LogSettings,
    configuration::{DataBaseSettings, Settings},
    domain::{Author, ClientId},
    routes::ingredient::{FormData, QueryData},
    startup::Application,
    telemetry::configure_tracing,
};
use once_cell::sync::Lazy;
use secrecy::{ExposeSecret, SecretString};
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
    pub api_token: AuthData,
}

pub enum Credentials {
    WithCredentials,
    NoCredentials,
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

    pub async fn post_author<Body>(
        &self,
        body: &Body,
        credentials: Credentials,
    ) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        let url = match credentials {
            Credentials::WithCredentials => &format!(
                "{}/author?api_key={}",
                &self.address,
                &self.api_token.api_key.expose_secret()
            ),
            Credentials::NoCredentials => &format!("{}/author", &self.address),
        };

        self.api_client
            .post(url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute post_author.")
    }

    pub async fn get_author(&self, author_id: &str, credentials: Credentials) -> reqwest::Response {
        let url = match credentials {
            Credentials::WithCredentials => &format!(
                "{}/author/{author_id}?api_key={}",
                &self.address,
                &self.api_token.api_key.expose_secret()
            ),
            Credentials::NoCredentials => &format!("{}/author/{author_id}", &self.address),
        };

        self.api_client
            .get(url)
            .send()
            .await
            .expect("Failed to execute get_author.")
    }

    pub async fn generate_access_token(&mut self) {
        self.api_token = generate_access_token(&self.db_pool)
            .await
            .expect("Failed to generate an API token for testing");
    }

    pub async fn patch_author(&self, body: &Author, credentials: Credentials) -> reqwest::Response {
        let url = match credentials {
            Credentials::WithCredentials => &format!(
                "{}/author/{}?api_key={}",
                &self.address,
                body.id().as_deref().unwrap(),
                &self.api_token.api_key.expose_secret()
            ),
            Credentials::NoCredentials => {
                &format!("{}/author/{}", &self.address, body.id().as_deref().unwrap())
            }
        };

        self.api_client
            .patch(url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute post_author.")
    }

    pub async fn options_author(&self) -> reqwest::Response {
        let url = format!("{}/author", &self.address);

        self.api_client
            .request(reqwest::Method::OPTIONS, url)
            .header("Access-Control-Request-Method", "GET")
            .send()
            .await
            .expect("Failed to send OPTIONS request.")
    }

    pub async fn head_author(&self, author_id: &str) -> reqwest::Response {
        let url = format!("{}/author/{author_id}", &self.address);

        self.api_client
            .head(url)
            .send()
            .await
            .expect("Failed to send OPTIONS request.")
    }

    pub async fn delete_author(
        &self,
        author_id: &str,
        credentials: Credentials,
    ) -> reqwest::Response {
        let url = match credentials {
            Credentials::WithCredentials => &format!(
                "{}/author/{author_id}?api_key={}",
                &self.address,
                &self.api_token.api_key.expose_secret()
            ),
            Credentials::NoCredentials => &format!("{}/author/{author_id}", &self.address),
        };

        self.api_client
            .delete(url)
            .send()
            .await
            .expect("Failed to execute post_author.")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Overwrite the DB name to provide a random name for every test runner. This way, we ensure that each test
    // is run in a pristine DB environment.
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

    let api_token = AuthData {
        api_key: SecretString::from("none"),
    };

    TestApp {
        address,
        db_pool,
        api_client,
        api_token,
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

// Seed an ApiUser in the test DB, and generate a token to grant access to the restricted endpoints during testing.
async fn generate_access_token(pool: &MySqlPool) -> Result<AuthData, anyhow::Error> {
    // Add a new entry in the ApiUser DB table.
    let client_id = ClientId::new();
    let query = sqlx::query!(
        r#"
        INSERT INTO ApiUser (id, email, validated,enabled,explanation) VALUES
        (?, ?, 1, 1, ?);
        "#,
        client_id.to_string(),
        "jane_doe@mail.com",
        "Because I'm testing this thing",
    );

    let token = SecretString::from(generate_token());
    let token_hashed = generate_new_token_hash(token.clone())?;

    // Save it into the DB.
    let mut transaction = pool
        .begin()
        .await
        .expect("Failed to start a new DB transaction");
    transaction
        .execute(query)
        .await
        .expect("Failed to create a dummy user for testing");
    store_validation_token(
        &mut transaction,
        &token_hashed,
        chrono::TimeDelta::days(1),
        &client_id,
    )
    .await?;
    transaction
        .commit()
        .await
        .expect("Failed to commit DB transaction");

    Ok(AuthData {
        api_key: SecretString::from(format!("{client_id}:{}", token.expose_secret())),
    })
}
