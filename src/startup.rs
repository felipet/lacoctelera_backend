//! Module that includes helper functions to start the **La Coctelera** application.

use crate::{
    configuration::{DataBaseSettings, Settings},
    routes,
};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::net::TcpListener;

pub struct Application {
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

        let server = run(
            listener,
            connection_pool,
            configuration.application.base_url,
        )
        .await?;

        Ok(Self { server })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
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
            .service(routes::echo)
            .service(routes::ingredient::get_ingredient)
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
        .connect_with(configuration.build_db_conn())
        .await
}
