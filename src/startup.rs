//! Module that includes helper functions to start the **La Coctelera** application.

use crate::{
    configuration::{DataBaseSettings, Settings},
    routes::{self, health},
    ApiDoc,
};
use actix_web::{dev::Server, web, App, HttpServer};
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

        let server = run(
            listener,
            connection_pool,
            configuration.application.base_url,
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
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(routes::echo)
            .service(health::options_echo)
            .service(health::health_check)
            .service(health::options_health)
            .service(routes::ingredient::get_ingredient)
            .service(routes::ingredient::add_ingredient)
            .service(routes::ingredient::options_ingredient)
            .service(routes::author::get_author)
            .service(routes::author::patch_author)
            .service(routes::author::head_author)
            .service(routes::author::options_author)
            .service(routes::recipe::get_recipe)
            .service(routes::recipe::search_recipe)
            .service(routes::recipe::options_recipe)
            .service(routes::recipe::head_recipe)
            .service(routes::recipe::post_recipe)
            .service(SwaggerUi::new("/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .app_data(db_pool.clone())
    })
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
