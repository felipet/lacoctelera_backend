// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Common stuff for running integration tests.

use actix_web::rt::spawn;
use lacoctelera::{
    authentication::{generate_new_token_hash, generate_token, store_validation_token, AuthData},
    configuration::{DataBaseSettings, LogSettings, Settings},
    domain::ClientId,
    startup::Application,
    telemetry::configure_tracing,
};
use once_cell::sync::Lazy;
use reqwest::Response;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Connection, Executor, MySqlConnection, MySqlPool};
use tracing::debug;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let mut settings = LogSettings {
        tracing_level: "info".into(),
        journald: Some(false),
        pretty_log: Some(true),
    };

    if std::env::var("TEST_LOG").is_ok() {
        let level = std::env::var("TEST_LOG").expect("Failed to read the content of TEST_LOG var");
        match level.as_str() {
            "info" => settings.tracing_level = "info".into(),
            "debug" => settings.tracing_level = "debug".into(),
            "warn" => settings.tracing_level = "warn".into(),
            "error" => settings.tracing_level = "error".into(),
            &_ => settings.tracing_level = "none".into(),
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Credentials {
    WithCredentials,
    NoCredentials,
}

impl From<bool> for Credentials {
    fn from(value: bool) -> Self {
        if value {
            Credentials::WithCredentials
        } else {
            Credentials::NoCredentials
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Resource {
    Ingredient,
    Recipe,
    Author,
    TokenRequest,
    TokenValidate,
}

impl From<&str> for Resource {
    fn from(value: &str) -> Resource {
        match value {
            "ingredient" => Resource::Ingredient,
            "author" => Resource::Author,
            "recipe" => Resource::Recipe,
            "token/request" => Resource::TokenRequest,
            "token/request/validate" => Resource::TokenValidate,
            _ => panic!("Wrong string given to make a Resource"),
        }
    }
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ss = match self {
            Resource::Ingredient => "ingredient",
            Resource::Author => "author",
            Resource::Recipe => "recipe",
            Resource::TokenRequest => "token/request",
            Resource::TokenValidate => "token/request/validate",
        };

        write!(f, "{}", ss)
    }
}

pub trait TestObject {
    async fn get(&self, query: &str) -> Response;
    async fn head(&self, id: &str) -> Response;
    async fn options(&self) -> Response;
    async fn post<Body: serde::Serialize>(&self, body: &Body) -> Response;
    async fn delete(&self, id: &str) -> Response;
    async fn patch<Body: serde::Serialize>(&self, id: &str, body: &Body) -> Response;
    async fn search(&self, query: &str) -> Response;
    fn db_pool(&self) -> &MySqlPool;
}

pub trait ApiTesterBuilder {
    type ApiTester;

    fn with_credentials(&mut self);
    fn without_credentials(&mut self);
    async fn build(self) -> Self::ApiTester;
}

pub struct TestBuilder;

impl TestBuilder {
    pub fn api_with_credentials(builder: &mut impl ApiTesterBuilder) {
        builder.with_credentials();
    }

    pub fn api_no_credentials(builder: &mut impl ApiTesterBuilder) {
        builder.without_credentials();
    }
}

impl TestApp {
    fn credentials_to_url(&self, credentials: Credentials) -> String {
        match credentials {
            Credentials::WithCredentials => {
                format!("?api_key={}", &self.api_token.api_key.expose_secret())
            }
            Credentials::NoCredentials => "".into(),
        }
    }

    pub async fn search_test(
        &self,
        target_resource: Resource,
        credentials: Credentials,
        query: &str,
    ) -> Response {
        let mut credential = self.credentials_to_url(credentials);
        if credentials == Credentials::WithCredentials {
            credential.remove(0);
            credential = format!("&{credential}");
        }

        let url = &format!("{}/{target_resource}{query}{credential}", &self.address);

        debug!("GET for /author using: {url}");

        self.api_client.get(url).send().await.expect(&format!(
            "Failed to execute GET for the resource {target_resource}."
        ))
    }

    pub async fn get_test(
        &self,
        target_resource: Resource,
        credentials: Credentials,
        query: &str,
    ) -> Response {
        let credentials = self.credentials_to_url(credentials);

        let url = &format!("{}/{target_resource}{query}{credentials}", &self.address);

        self.api_client.get(url).send().await.expect(&format!(
            "Failed to execute GET for the resource {target_resource}."
        ))
    }

    pub async fn post_test<Body>(
        &self,
        target_resource: Resource,
        credentials: Credentials,
        body: &Body,
    ) -> Response
    where
        Body: serde::Serialize,
    {
        let credentials = self.credentials_to_url(credentials);

        match target_resource {
            Resource::TokenRequest => self
                .api_client
                .post(&format!("{}/{target_resource}{credentials}", &self.address))
                .form(body)
                .header("Content-type", "application/json")
                .send()
                .await
                .expect(&format!(
                    "Failed to execute POST for the resource {target_resource}."
                )),
            _ => self
                .api_client
                .post(&format!("{}/{target_resource}{credentials}", &self.address))
                .json(body)
                .header("Content-type", "application/json")
                .send()
                .await
                .expect(&format!(
                    "Failed to execute POST for the resource {target_resource}."
                )),
        }
    }

    pub async fn head_test(&self, target_resource: Resource, id: &str) -> Response {
        let url = format!("{}/{target_resource}/{id}", &self.address);

        self.api_client.head(url).send().await.expect(&format!(
            "Failed to execute HEAD for the resource {target_resource}."
        ))
    }

    pub async fn delete_test(
        &self,
        target_resource: Resource,
        credentials: Credentials,
        id: &str,
    ) -> Response {
        let credentials = self.credentials_to_url(credentials);

        let url = format!("{}/{target_resource}/{id}{credentials}", &self.address);

        self.api_client.delete(url).send().await.expect(&format!(
            "Failed to execute HEAD for the resource {target_resource}."
        ))
    }

    pub async fn patch_test<Body>(
        &self,
        target_resource: Resource,
        credentials: Credentials,
        id: &str,
        body: &Body,
    ) -> Response
    where
        Body: serde::Serialize,
    {
        let credentials = self.credentials_to_url(credentials);
        let url = format!("{}/{target_resource}/{id}{credentials}", &self.address);

        self.api_client
            .patch(url)
            .json(body)
            .send()
            .await
            .expect("Failed to execute PATCH for the resource {target_resource}.")
    }

    pub async fn options_test(&self, target_resource: Resource) -> Response {
        let url = format!("{}/{target_resource}", &self.address);

        self.api_client
            .request(reqwest::Method::OPTIONS, url)
            .header("Access-Control-Request-Method", "GET")
            .send()
            .await
            .expect(&format!(
                "Failed to execute HEAD for the resource {target_resource}."
            ))
    }

    pub async fn post_token_request<Body>(&self, body: &Body) -> Response
    where
        Body: serde::Serialize,
    {
        self.post_test(Resource::TokenRequest, Credentials::NoCredentials, body)
            .await
    }

    pub async fn generate_access_token(&mut self) {
        self.api_token = generate_access_token(&self.db_pool)
            .await
            .expect("Failed to generate an API token for testing");
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
    let address = format!(
        "http://{}:{port}{}/v{}",
        configuration.application.host,
        configuration.application.base_url,
        env!("CARGO_PKG_VERSION").split(".").collect::<Vec<&str>>()[0]
    );
    let _ = spawn(application.run_until_stopped());

    // Instantiate an HTTP client to run the tests against the app's backend.
    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
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
