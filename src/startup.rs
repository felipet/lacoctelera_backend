// Copyright 2024 Felipe Torres González
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Module that includes helper functions to start the **La Coctelera** application.

use crate::{
    configuration::{DataBaseSettings, Settings},
    routes::{self, health},
    ApiDoc,
};
use actix_files as fs;
use actix_web::{dev::Server, web, App, HttpServer};
use mailjet_client::{MailjetClient, MailjetClientBuilder};
use secrecy::ExposeSecret;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        // Create a connection pool to handle connections to the DB.
        let connection_pool = get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connect to MariaDB.");

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let max_workers = configuration.application.max_workers;

        let mut mail_client = MailjetClientBuilder::new(
            configuration.email_client.api_user,
            configuration.email_client.api_key,
        )
        .with_api_version(&configuration.email_client.target_api)
        .with_email_name("La Coctelera")
        .with_email_address(configuration.email_client.admin_address.expose_secret())
        .with_https_enforcing(true)
        .build()?;

        if configuration.email_client.sandbox_mode.unwrap_or_default() {
            mail_client.enable_sandbox_mode();
        }

        let server = run(
            listener,
            connection_pool,
            configuration.application.base_url,
            max_workers,
            mail_client,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

pub async fn run(
    listener: TcpListener,
    db_pool: MySqlPool,
    _base_url: String,
    max_workers: u16,
    mail_client: MailjetClient,
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);
    let mail_client = web::Data::new(mail_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(routes::echo)
            .service(health::options_echo)
            .service(health::health_check)
            .service(health::options_health)
            .service(routes::ingredient::search_ingredient)
            .service(routes::ingredient::add_ingredient)
            .service(routes::ingredient::options_ingredient)
            .service(routes::author::search_author)
            .service(routes::author::patch_author)
            .service(routes::author::head_author)
            .service(routes::author::options_author)
            .service(routes::author::post_author)
            .service(routes::author::get_author)
            .service(routes::author::delete_author)
            .service(routes::recipe::get_recipe)
            .service(routes::recipe::search_recipe)
            .service(routes::recipe::options_recipe)
            .service(routes::recipe::head_recipe)
            .service(routes::recipe::post_recipe)
            .service(fs::Files::new("/static", "./static/resources").show_files_listing())
            .service(
                web::scope("/token")
                    .service(routes::token::token_req_get)
                    .service(routes::token::token_req_post)
                    .service(routes::token::req_validation),
            )
            .service(SwaggerUi::new("/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .app_data(db_pool.clone())
            .app_data(mail_client.clone())
    })
    .workers(max_workers as usize)
    .listen(listener)?
    .run();

    Ok(server)
}

pub async fn get_connection_pool(
    configuration: &DataBaseSettings,
) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(configuration.max_connections as u32)
        .idle_timeout(configuration.idle_timeout())
        .connect_with(configuration.build_db_conn_with_db())
        .await
}
